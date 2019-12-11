use core::fmt;

pub type TockResult<T> = Result<T, TockError>;

#[derive(Copy, Clone)]
pub enum TockError {
    Subscribe(SubscribeError),
    Command(CommandError),
    Allow(AllowError),
    Format,
    Other(OtherError),
}

#[derive(Copy, Clone)]
pub struct SubscribeError {
    pub driver_number: usize,
    pub subscribe_number: usize,
    pub return_code: isize,
}

impl From<SubscribeError> for TockError {
    fn from(subscribe_error: SubscribeError) -> Self {
        TockError::Subscribe(subscribe_error)
    }
}

#[derive(Copy, Clone)]
pub struct CommandError {
    pub driver_number: usize,
    pub command_number: usize,
    pub arg1: usize,
    pub arg2: usize,
    pub return_code: isize,
}

impl From<CommandError> for TockError {
    fn from(command_error: CommandError) -> Self {
        TockError::Command(command_error)
    }
}

#[derive(Copy, Clone)]
pub struct AllowError {
    pub driver_number: usize,
    pub allow_number: usize,
    pub return_code: isize,
}

impl From<AllowError> for TockError {
    fn from(allow_error: AllowError) -> Self {
        TockError::Allow(allow_error)
    }
}

impl From<fmt::Error> for TockError {
    fn from(fmt::Error: fmt::Error) -> Self {
        TockError::Format
    }
}

#[derive(Copy, Clone)]
pub enum OtherError {
    TimerDriverDurationOutOfRange,
    TimerDriverErroneousClockFrequency,
}

impl From<OtherError> for TockError {
    fn from(other: OtherError) -> Self {
        TockError::Other(other)
    }
}

pub const SUCCESS: isize = 0;
pub const FAIL: isize = -1;
pub const EBUSY: isize = -2;
pub const EALREADY: isize = -3;
pub const EINVAL: isize = -6;
pub const ESIZE: isize = -7;
pub const ENOMEM: isize = -9;
