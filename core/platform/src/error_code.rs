/// A system call error code. This can either be an error code returned by the
/// kernel or BADRVAL, which indicates the kernel returned the wrong type of
/// response to a system call.
// ErrorCode is not an enum so that conversion from the kernel's return value (a
// `usize` in a register) is free.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ErrorCode {
    value: usize,
}

impl From<usize> for ErrorCode {
    fn from(value: usize) -> ErrorCode {
        ErrorCode { value }
    }
}

impl From<ErrorCode> for usize {
    fn from(error_code: ErrorCode) -> usize {
        error_code.value
    }
}

pub const FAIL: ErrorCode = ErrorCode { value: 1 };
pub const BUSY: ErrorCode = ErrorCode { value: 2 };
pub const ALREADY: ErrorCode = ErrorCode { value: 3 };
pub const OFF: ErrorCode = ErrorCode { value: 4 };
pub const RESERVE: ErrorCode = ErrorCode { value: 5 };
pub const INVALID: ErrorCode = ErrorCode { value: 6 };
pub const SIZE: ErrorCode = ErrorCode { value: 7 };
pub const CANCEL: ErrorCode = ErrorCode { value: 8 };
pub const NOMEM: ErrorCode = ErrorCode { value: 9 };
pub const NOSUPPORT: ErrorCode = ErrorCode { value: 10 };
pub const NODEVICE: ErrorCode = ErrorCode { value: 11 };
pub const UNINSTALLED: ErrorCode = ErrorCode { value: 12 };
pub const NOACK: ErrorCode = ErrorCode { value: 13 };
pub const BADRVAL: ErrorCode = ErrorCode { value: 1024 };
