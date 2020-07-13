/// An error code returned by the kernel. Tock's system calls return errors as a
/// negative `isize`. This wraps the isize, and is useful for adding type safety
/// to APIs.

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ErrorCode {
    // Note: value *should* always be negative, but we do not *verify* that so
    // unsafe code cannot rely on value being negative.
    value: isize,
}

impl ErrorCode {
    // Converts the given isize into an ErrorCode. Note that the isize should be
    // negative, although that is not verified to reduce code size. We don't
    // implement From because not every isize converts sensibly to an ErrorCode.
    pub fn new(value: isize) -> ErrorCode {
        ErrorCode { value }
    }
}

impl Into<isize> for ErrorCode {
    fn into(self) -> isize {
        self.value
    }
}
