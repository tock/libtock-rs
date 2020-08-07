/// `AllowedSlice` is a slice-based analogue of `Allowed`. It represents a slice
/// that has been shared with the kernel using the `allow` system call.
/// Unlike `Allowed`, `AllowSlice`'s methods accept an index into the shared
/// slice, and operate on the element at that index.
// Like Allowed, AllowedSlice does not directly use the 'b lifetime. Platform
// uses 'b to prevent the AllowedSlice from accessing the buffer after the
// buffer becomes invalid.
// AllowedSlice requires T to be Copy for the same reasons as Allowed. See
// Allowed's comments for more information.
pub struct AllowedSlice<'b, T: Copy + 'b> {
    // `data` points to the start of the shared slice, and `len` is the length
    // of that slice.
    // Safety properties:
    //   1. The slice remains valid and usable for the lifetime of this
    //      AllowedSlice instance.
    //   2. Read and write accesses into the slice must be performed as a
    //      volatile operation, as the kernel may mutate the slice at any time.
    //   3. The shared slice may have an arbitrary bit pattern in it, so reading
    //      from the slice is only safe if the type contained within is
    //      AllowReadable.
    data: core::ptr::NonNull<T>,
    len: usize,

    // Use the 'b parameter, and make AllowedSlice invariant w.r.t. T. See
    // Allowed's comment for a more detailed description.
    _phantom: core::marker::PhantomData<&'b mut [T]>,
}

// AllowedSlice's API mirrors that of Allowed, but with indexing on most
// methods. Se Allowed's comments for more details.
impl<'b, T: Copy + 'b> AllowedSlice<'b, T> {
    // The caller (Platform) must make sure the following are true:
    // 1. `data` points to a valid [T] slice of length `len`.
    // 2. There are no other references to the slice.
    // 3. The slice remains usable until the AllowedSlice's lifetime has ended.
    #[allow(unused)] // TODO: Remove when Platform is implemented.
    pub(crate) unsafe fn new(data: core::ptr::NonNull<T>, len: usize) -> AllowedSlice<'b, T> {
        AllowedSlice {
            data,
            len,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Sets the value at `index`, or does nothing if `index` is out of range.
    pub fn set(&self, index: usize, value: T) -> Result<(), OutOfBounds> {
        if index >= self.len {
            return Err(OutOfBounds);
        }
        unsafe {
            core::ptr::write_volatile(self.data.as_ptr().add(index), value);
        }
        Ok(())
    }
}

impl<'b, T: crate::AllowReadable + Copy + 'b> AllowedSlice<'b, T> {
    /// Returns the value at `index` without changing it.
    pub fn get(&self, index: usize) -> Result<T, OutOfBounds> {
        if index >= self.len {
            return Err(OutOfBounds);
        }
        Ok(unsafe { core::ptr::read_volatile(self.data.as_ptr().add(index)) })
    }

    /// Returns the value at `index` without changing it. If `index` is out of
    /// bounds, returns the provided default value.
    pub fn get_or_default(&self, index: usize, default: T) -> T {
        if index >= self.len {
            return default;
        }
        unsafe { core::ptr::read_volatile(self.data.as_ptr().add(index)) }
    }
}

/// An error type indicating an out-of-bounds access.
#[derive(PartialEq)]
pub struct OutOfBounds;
