use core::mem::transmute;

/// In order to work with Miri's `-Zmiri-track-raw-pointers` flag, we cannot
/// pass pointers to the kernel through `usize` values (as casting to and from
/// `usize` drops the pointer`s tag). Instead, `RawSyscalls` uses the `Register`
/// type. `Register` wraps a raw pointer type that keeps that tags around. User
/// code should not depend on the particular type of pointer that `Register`
/// wraps, but instead use the conversion functions in this module.
// Register is repr(transparent) so that an upcall's application data can be
// soundly passed as a Register.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct Register(pub *mut ());

// -----------------------------------------------------------------------------
// Conversions to Register
// -----------------------------------------------------------------------------

impl From<crate::ErrorCode> for Register {
    fn from(value: crate::ErrorCode) -> Register {
        (value as usize).into()
    }
}

impl From<u32> for Register {
    fn from(value: u32) -> Register {
        (value as usize).into()
    }
}

impl From<i32> for Register {
    fn from(value: i32) -> Register {
        (value as usize).into()
    }
}

impl From<usize> for Register {
    fn from(value: usize) -> Register {
        // Note: clippy is wrong here; transmute has different semantics than
        // `as` casts under strict provenance.
        #[allow(clippy::useless_transmute)]
        // We want to convert using the same semantics as core::ptr::invalid:
        // convert the usize into a pointer with that address without attaching
        // provenance to it. However, core::ptr::invalid is a nightly-only
        // function. In order to build on stable, we copy its implementation.
        // Safety: Raw pointers do not have any validity invariants that usize
        // does not have; a raw pointer can point to any address.
        Register(unsafe { transmute(value) })
    }
}

impl<T> From<*mut T> for Register {
    fn from(value: *mut T) -> Register {
        Register(value as *mut ())
    }
}

impl<T> From<*const T> for Register {
    fn from(value: *const T) -> Register {
        Register(value as *mut ())
    }
}

// -----------------------------------------------------------------------------
// Infallible conversions from Register
// -----------------------------------------------------------------------------

// If we implement From<u32> on Register, then we automatically get a
// TryFrom<Error = Infallible> implementation, which conflicts with our fallible
// TryFrom implementation. We could choose to not implement TryFrom and instead
// add a fallible accessor (something like "expect_u32"), but that seems
// confusing. Instead, we use an inherent method for the Register -> u32
// infallible conversion.
impl Register {
    /// Casts this register to a u32, truncating it if it is larger than
    /// u32::MAX. This conversion should be avoided in host-based test code; use
    /// the `TryFrom<Register> for u32` implementation instead.
    pub fn as_u32(self) -> u32 {
        self.0 as u32
    }

    /// Casts this register to a i32, truncating it if it is larger than
    /// 32 bits. This conversion should be avoided in host-based test code; use
    /// the `TryFrom<Register> for i32` implementation instead.
    pub fn as_i32(self) -> i32 {
        self.0 as i32
    }
}

impl From<Register> for usize {
    fn from(register: Register) -> usize {
        register.0 as usize
    }
}

impl<T> From<Register> for *mut T {
    fn from(register: Register) -> *mut T {
        register.0 as *mut T
    }
}

impl<T> From<Register> for *const T {
    fn from(register: Register) -> *const T {
        register.0 as *const T
    }
}

// -----------------------------------------------------------------------------
// Fallible conversions from Register
// -----------------------------------------------------------------------------

/// Converts a `Register` to a `u32`. Returns an error if the `Register`'s value
/// is larger than `u32::MAX`. This is intended for use in host-based tests; in
/// Tock process binary code, use Register::as_u32 instead.
impl TryFrom<Register> for u32 {
    type Error = core::num::TryFromIntError;

    fn try_from(register: Register) -> Result<u32, core::num::TryFromIntError> {
        (register.0 as usize).try_into()
    }
}
