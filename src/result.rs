use core::fmt::Debug;
use core::fmt::Error;
use core::fmt::Formatter;

#[derive(Copy, Clone)]
pub struct TockError(pub(crate) isize);

pub type TockResult<T> = Result<T, TockError>;

impl TockError {
    pub fn from_return_code(return_code: isize) -> TockResult<usize> {
        if return_code >= 0 {
            Ok(return_code as usize)
        } else {
            Err(TockError(return_code))
        }
    }

    pub fn get_return_code(&self) -> isize {
        self.0
    }
}

impl Debug for TockError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        match self.0 {
            FAIL => write!(formatter, "FAIL"),
            EBUSY => write!(formatter, "EBUSY"),
            EALREADY => write!(formatter, "EALREADY"),
            EOFF => write!(formatter, "EOFF"),
            ERESERVE => write!(formatter, "ERESERVE"),
            EINVAL => write!(formatter, "EINVAL"),
            ESIZE => write!(formatter, "ESIZE"),
            ECANCEL => write!(formatter, "ECANCEL"),
            ENOMEM => write!(formatter, "ENOMEM"),
            ENOSUPPORT => write!(formatter, "ENOSUPPORT"),
            ENODEVICE => write!(formatter, "ENODEVICE"),
            EUNINSTALLED => write!(formatter, "EUNINSTALLED"),
            ENOACK => write!(formatter, "ENOACK"),
            unknown => write!(formatter, "Unknown error code: {}", unknown),
        }
    }
}

pub const SUCCESS: isize = 0;
pub const FAIL: isize = -1;
pub const EBUSY: isize = -2;
pub const EALREADY: isize = -3;
pub const EOFF: isize = -4;
pub const ERESERVE: isize = -5;
pub const EINVAL: isize = -6;
pub const ESIZE: isize = -7;
pub const ECANCEL: isize = -8;
pub const ENOMEM: isize = -9;
pub const ENOSUPPORT: isize = -10;
pub const ENODEVICE: isize = -11;
pub const EUNINSTALLED: isize = -12;
pub const ENOACK: isize = -13;
