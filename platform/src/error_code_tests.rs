// Verifies that `ErrorCode` represents every valid value in the range
// [1, 1023].
#[cfg(miri)]
#[test]
fn error_code_range() {
    for value in 1..=1023u16 {
        unsafe { *(&value as *const u16 as *const crate::ErrorCode) };
    }
}
