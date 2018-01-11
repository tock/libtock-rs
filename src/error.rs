#[derive(Copy, Clone, Debug)]
pub enum TockError<E> {
    Expected(E),
    Unexpected(isize),
}

pub type TockResult<T, E> = Result<T, TockError<E>>;

pub trait TockResultExt<T, E>: Sized {
    fn as_expected(self) -> Result<T, E>;
}

impl<T, E> TockResultExt<T, E> for TockResult<T, E> {
    fn as_expected(self) -> Result<T, E> {
        match self {
            Ok(ok) => Ok(ok),
            Err(TockError::Expected(err)) => Err(err),
            Err(TockError::Unexpected(_)) => panic!("Unexpected error"),
        }
    }
}

pub const SUCCESS: isize = 0;
pub const ENOMEM: isize = -9;
