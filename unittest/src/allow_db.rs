use core::num::NonZeroUsize;
use libtock_platform::Register;

/// `AllowDb` stores the currently-active Allow buffers, and is responsible for
/// preventing overlapping Allow buffers.
///
/// Functionally, it receives raw register data from the Allow system calls
/// implementations, verifies the new buffer does not overlap any active buffer,
/// and converts the register values into usable reference types (such as
/// `&'static mut [u8]`). When a buffer reference is returned, it removes it
/// from its database and returns the raw register values.
// TRD 104 (Tock's system call ABI) says that allow buffers only overlap if they
// have a memory address in common, so zero-sized buffers cannot overlap.
//
// Read-Write Allow and Read-Only Allow are invoked through
// RawSyscalls::syscall4, which is unsafe, and requires its caller to pass
// arguments that are valid for the system call. Those requirements require that
// either the length field is zero, or the address and length field represent a
// valid slice. Several of the steps in this file require that property.
//
// Therefore AllowDb does not need to check for overlaps with zero-sized
// buffers.
#[derive(Default)]
pub struct AllowDb {
    // List of all active buffers, excluding zero-sized buffers. Contains both
    // read-only buffers and read-write buffers.
    // Key: address of the buffer.
    // Value: length of the buffer.
    // Invariant: These buffers never overlap, and represent valid slices
    // (although they can't be converted to slices because fake drivers may have
    // a &mut [u8] pointing at the buffer).
    buffers: std::collections::BTreeMap<*mut u8, NonZeroUsize>,
}

impl AllowDb {
    // Adds a new buffer, or returns an error if it overlaps with any existing
    // buffers. Requires that address and len represent a valid slice.
    unsafe fn insert_raw(
        &mut self,
        address: *mut u8,
        len: NonZeroUsize,
    ) -> Result<(), OverlapError> {
        // The new buffer spans the address range [address, address + len - 1],
        // so the highest-address buffer it overlaps with starts at
        // address + len - 1. It can overlap with a buffer that starts with any
        // address less than that (if that buffer's length is large enough), so
        // we don't give a lower bound for the range.
        //
        // If the last buffer in this range does NOT overlap the new buffer,
        // then that buffer's memory range is strictly less than the new buffer.
        // Because buffers in self.buffers cannot overlap, all of the buffers in
        // the range would need to be strictly less than the new buffer.
        // Therefore we only need to check for overlap with the last buffer in
        // this range.
        //
        // range_end is one past the end of the range to check.
        let range_end = unsafe {
            // Safety: The function's preconditions require that address and len
            // represent a valid slice, which guarantees that this sum does not
            // oveflow.
            address.add(len.get())
        };
        if let Some(existing_buffer) = self.buffers.range(..range_end).next_back() {
            let (&existing_address, &existing_len) = existing_buffer;
            // Check if existing_buffer overlaps with the new buffer. Note that
            // existing_address.add(existing_len) generates a pointer one past
            // the end of the existing buffer, which is why the inequality is
            // strict.
            // Safety: self.buffers has an invariant that its values represent
            // slices, so this sum cannot overflow.
            if unsafe { existing_address.add(existing_len.get()) } > address {
                return Err(OverlapError);
            }
        }
        self.buffers.insert(address, len);
        Ok(())
    }

    /// Adds a read-only buffer to the database, and returns it as a
    /// `RoAllowBuffer`.
    ///
    /// # Safety
    /// `address` and `len` must be valid as specified in TRD 104: either `len`
    /// is 0 or `address` and `len` represent a valid slice.
    pub unsafe fn insert_ro_buffer(
        &mut self,
        address: Register,
        len: Register,
    ) -> Result<RoAllowBuffer, OverlapError> {
        let len: usize = len.into();
        if let Some(nonzero_len) = NonZeroUsize::new(len) {
            // The buffer is not zero-sized. Add it to the database (checking it
            // does not overlap an existing buffer).
            // Safety: `len` is nonzero, so by this function's precondition
            // `address` and `len` represent a valid slice.
            unsafe { self.insert_raw(address.into(), nonzero_len) }?;
        }
        Ok(RoAllowBuffer {
            address: address.into(),
            len,
        })
    }

    /// Adds a read-write buffer to the database, and returns it as a
    /// `RwAllowBuffer`.
    ///
    /// # Safety
    /// `address` and `len` must be valid as specified in TRD 104: either `len`
    /// is 0 or `address` and `len` represent a valid slice.
    pub unsafe fn insert_rw_buffer(
        &mut self,
        address: Register,
        len: Register,
    ) -> Result<RwAllowBuffer, OverlapError> {
        let address: *mut u8 = address.into();
        let len: usize = len.into();
        if let Some(nonzero_len) = NonZeroUsize::new(len) {
            // The buffer is not zero-sized. Add it to the database (checking it
            // does not overlap an existing buffer).
            // Safety: `len` is nonzero, so by this function's precondition
            // `address` and `len` represent a valid slice.
            unsafe { self.insert_raw(address, nonzero_len) }?;
        }
        Ok(RwAllowBuffer { address, len })
    }

    /// Removes a read-only buffer from the database and returns its raw
    /// register values.
    ///
    /// The returned value is the tuple (address, len) passed into the
    /// insert_ro_buffer call that created the RoAllowBuffer.
    pub fn remove_ro_buffer(&mut self, buffer: RoAllowBuffer) -> (Register, Register) {
        self.buffers.remove(&(buffer.address as *mut u8));
        (buffer.address.into(), buffer.len.into())
    }

    /// Removes a read-write buffer from the database and returns its raw
    /// register values.
    ///
    /// The returned value is the tuple (address, len) passed into the
    /// insert_rw_buffer call that created the RwAllowBuffer.
    pub fn remove_rw_buffer(&mut self, buffer: RwAllowBuffer) -> (Register, Register) {
        self.buffers.remove(&buffer.address);
        (buffer.address.into(), buffer.len.into())
    }
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
#[error("allow buffers overlap")]
pub struct OverlapError;

/// A read-only reference to a buffer that has been shared via the Allow system
/// call. This reference is non-Copy, so `AllowDb` can determine when all
/// references to the buffer have been destroyed.
#[derive(Debug)]
pub struct RoAllowBuffer {
    // Safety invariant: Either length is 0, or address and length can be
    // soundly converted to a &'static [u8]. Note: that means that no &mut [u8]
    // references may overlap the slice described by address and len.
    address: *const u8,
    len: usize,
}

impl Default for RoAllowBuffer {
    fn default() -> RoAllowBuffer {
        RoAllowBuffer {
            address: core::ptr::null(),
            len: 0,
        }
    }
}

// Allows access to the pointed-to-buffer. The returned reference has the same
// lifetime as the &self reference, so the caller can't keep the reference for
// longer than it has access to the RoAllowBuffer.
impl std::ops::Deref for RoAllowBuffer {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self.len {
            0 => &[],
            // Safety: Because length is nonzero, the safety invariant on
            // address and len says this conversion is sound.
            _ => unsafe { core::slice::from_raw_parts(self.address, self.len) },
        }
    }
}

/// A read-write reference to a buffer that has been shared via the Allow system
/// call. This reference is non-Copy, so `AllowDb` can determine when all
/// references to the buffer have been destroyed.
#[derive(Debug)]
pub struct RwAllowBuffer {
    // Safety invariant: Either length is 0, or address and length can be
    // soundly converted to a &'static mut [u8]. Note: that means that no
    // references may overlap the slice described by address and len.
    address: *mut u8,
    len: usize,
}

impl Default for RwAllowBuffer {
    fn default() -> RwAllowBuffer {
        RwAllowBuffer {
            address: core::ptr::null_mut(),
            len: 0,
        }
    }
}

// Allows access to the pointed-to-buffer. The returned reference has the same
// lifetime as the &self reference, so the caller can't keep the reference for
// longer than it has access to the RwAllowBuffer.
impl std::ops::Deref for RwAllowBuffer {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self.len {
            0 => &[],
            // Safety: Because length is nonzero, the safety invariant on
            // address and len says this conversion is sound.
            _ => unsafe { core::slice::from_raw_parts(self.address, self.len) },
        }
    }
}

// Same purpose as the Deref implementation, but for mut references.
impl std::ops::DerefMut for RwAllowBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        match self.len {
            0 => &mut [],
            // Safety: Because length is nonzero, the safety invariant on
            // address and len says this conversion is sound.
            _ => unsafe { core::slice::from_raw_parts_mut(self.address, self.len) },
        }
    }
}
