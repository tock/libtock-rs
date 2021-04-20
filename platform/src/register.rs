/// In order to work with Miri's `-Zmiri-track-raw-pointers` flag, we cannot
/// pass pointers to the kernel through `usize` values (as casting to and from
/// `usize` drops the pointer`s tag). Instead, `RawSyscalls`
/// uses the `Register` type. `Register` is an alias for a raw pointer type that
/// keeps that tags around. User code should not depend on the particular type
/// of pointer that `Register` aliases, but instead use the conversion functions
/// in this module.
pub type Register = *mut ();

// -----------------------------------------------------------------------------
// Conversions to Register
// -----------------------------------------------------------------------------

pub fn from_u32(value: u32) -> Register {
    value as Register
}

pub fn from_usize(value: usize) -> Register {
    value as Register
}

pub fn from_mut_ptr<T>(value: *mut T) -> Register {
    value as Register
}

pub fn from_ptr<T>(value: *const T) -> Register {
    value as Register
}

// -----------------------------------------------------------------------------
// Infallable conversions from Register
// -----------------------------------------------------------------------------

pub fn as_u32(register: Register) -> u32 {
    register as u32
}

pub fn as_usize(register: Register) -> usize {
    register as usize
}

pub fn as_mut_ptr<T>(register: Register) -> *mut T {
    register as *mut T
}

pub fn as_ptr<T>(register: Register) -> *const T {
    register as *const T
}

// -----------------------------------------------------------------------------
// Fallable conversions from Register
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooLargeError(());

pub fn try_into_u32(register: Register) -> Result<u32, TooLargeError> {
    use core::convert::TryInto;
    (register as usize)
        .try_into()
        .map_err(|_| TooLargeError(()))
}
