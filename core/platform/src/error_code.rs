/// A system call error code. This can either be an error code returned by the
/// kernel or BADRVAL, which indicates the kernel returned the wrong type of
/// response to a system call.
// ErrorCode is not an enum so that conversion from the kernel's return value (a
// `usize` in a register) is free.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ErrorCode(usize);

impl From<usize> for ErrorCode {
    fn from(value: usize) -> ErrorCode {
        ErrorCode(value)
    }
}

impl From<ErrorCode> for usize {
    fn from(error_code: ErrorCode) -> usize {
        error_code.0
    }
}

pub const FAIL: ErrorCode = ErrorCode(1);
pub const BUSY: ErrorCode = ErrorCode(2);
pub const ALREADY: ErrorCode = ErrorCode(3);
pub const OFF: ErrorCode = ErrorCode(4);
pub const RESERVE: ErrorCode = ErrorCode(5);
pub const INVALID: ErrorCode = ErrorCode(6);
pub const SIZE: ErrorCode = ErrorCode(7);
pub const CANCEL: ErrorCode = ErrorCode(8);
pub const NOMEM: ErrorCode = ErrorCode(9);
pub const NOSUPPORT: ErrorCode = ErrorCode(10);
pub const NODEVICE: ErrorCode = ErrorCode(11);
pub const UNINSTALLED: ErrorCode = ErrorCode(12);
pub const NOACK: ErrorCode = ErrorCode(13);
pub const BADRVAL: ErrorCode = ErrorCode(1024);
