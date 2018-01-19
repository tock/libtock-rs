#[derive(Copy, Clone, Debug)]
pub enum TockValue<E> {
    Expected(E),
    Unexpected(isize),
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
pub const ENOMEM: isize = -9;
