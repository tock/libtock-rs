#[derive(Copy, Clone)]
pub enum TockValue<E> {
    Expected(E),
    Unexpected(isize),
}

// Size-optimized implementation of Debug.
impl<E: core::fmt::Debug> core::fmt::Debug for TockValue<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self {
            // Printing out the value of E would cause TockResult::unwrap() to
            // use a &dyn core::fmt::Debug, which defeats LLVM's
            // devirtualization and prevents LTO from removing unused Debug
            // implementations. Unfortunately, that generates too much code
            // bloat (several kB), so we cannot display the value contained in
            // this TockValue.
            TockValue::Expected(_) => f.write_str("Expected(...)"),

            TockValue::Unexpected(n) => {
                f.write_str("Unexpected(")?;
                n.fmt(f)?;
                f.write_str(")")
            }
        }
    }
}

pub type TockResult<T, E> = Result<T, TockValue<E>>;

pub trait TockResultExt<T, E>: Sized {
    fn as_expected(self) -> Result<T, E>;
}

impl<T, E> TockResultExt<T, E> for TockResult<T, E> {
    fn as_expected(self) -> Result<T, E> {
        match self {
            Ok(ok) => Ok(ok),
            Err(TockValue::Expected(err)) => Err(err),
            Err(TockValue::Unexpected(_)) => panic!("Unexpected error"),
        }
    }
}

pub const SUCCESS: isize = 0;
pub const FAIL: isize = -1;
pub const EBUSY: isize = -2;
pub const EALREADY: isize = -3;
pub const EINVAL: isize = -6;
pub const ESIZE: isize = -7;
pub const ENOMEM: isize = -9;
