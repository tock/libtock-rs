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
    pub return_code: ReturnCode,
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
    pub return_code: ReturnCode,
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
    pub return_code: ReturnCode,
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ReturnCode {
    SUCCESS,
    FAIL,
    EBUSY,
    EALREADY,
    EINVAL,
    ESIZE,
    ENOMEM,
    UNKNOWN(isize),
}

impl From<isize> for ReturnCode {
    fn from(return_code: isize) -> Self {
        match return_code {
            0 => ReturnCode::SUCCESS,
            -1 => ReturnCode::FAIL,
            -2 => ReturnCode::EBUSY,
            -3 => ReturnCode::EALREADY,
            -6 => ReturnCode::EINVAL,
            -7 => ReturnCode::ESIZE,
            -9 => ReturnCode::ENOMEM,
            _ => ReturnCode::UNKNOWN(return_code),
        }
    }
}
