use core::convert::TryInto;

use crate::{error_code::NotAnErrorCode, ErrorCode};

// Verifies that `ErrorCode` represents every valid value in the range
// [1, 1024].
#[cfg(miri)]
#[test]
fn error_code_range() {
    for value in 1..=1024u32 {
        let _ = unsafe { *(&value as *const u32 as *const ErrorCode) };
    }
}

#[test]
fn error_code_try_into() {
    assert_eq!(TryInto::<ErrorCode>::try_into(0u32), Err(NotAnErrorCode));
    for value in 1..=1024u32 {
        assert_eq!(value.try_into().map(|e: ErrorCode| e as u32), Ok(value));
    }
    assert_eq!(TryInto::<ErrorCode>::try_into(1025u32), Err(NotAnErrorCode));
}
