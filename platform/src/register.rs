/// `Register` represents the value of a register used in Tock's syscall ABI. It
/// can contain integer values as well as pointer values.
///
/// `Register` currently wraps a raw pointer, but that is not a stable guarantee
/// and users should not rely on it. However, `Register` does guarantee that the
/// type it wraps is a valid operand type for inline assembly.
///
/// If a pointer is converted to a `Register`, that `Register` has that
/// pointer's provenance. The provenance is not exposed. If an integer is
/// converted to a `Register`, that `Register` has no provenance. When a
/// `Register` with provenance is converted into a pointer, that pointer carries
/// the `Register`'s provenance. When a `Register` without provenance is
/// converted into a pointer, that pointer has no provenance.
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
        // TODO(Rust 1.84): We want to convert using the same semantics as
        // core::ptr::without_provenance. However, until our MSRV is >= 1.84, we
        // have to use an `as` cast instead. This may result in this Register
        // converting into a pointer with provenance later on, but that
        // shouldn't break any users of Register in practice.
        #[cfg(not(miri))]
        {
            Register(value as *mut ())
        }
        // However, on Miri, we cannot do the conversion using an `as` cast.
        // Fortunately, since Miri runs on nightly Rust, we can use
        // `without_provenance_mut`.
        #[cfg(miri)]
        {
            Register(core::ptr::without_provenance_mut(value))
        }
    }
}

impl<T> From<*mut T> for Register {
    fn from(value: *mut T) -> Register {
        Register(value.cast())
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
        // TODO(Rust 1.84): We want to convert using the same semantics as
        // .addr(). Until our MSRV is >= 1.84, we have to convert using an `as`
        // cast instead. This exposes the provenance of the pointer, which is
        // not correct but shouldn't break any users in practice.
        register.0 as usize
    }
}

impl<T> From<Register> for *mut T {
    fn from(register: Register) -> *mut T {
        register.0.cast()
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
        usize::from(register).try_into()
    }
}
