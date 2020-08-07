// TODO: Add unit tests

use crate::ErrorCode;

/// ReturnCode is a lightweight wrapper around the kernel's return values. It is
/// the size of an `isize` but contains methods expressing the semantics of the
/// kernel's ReturnCode (see
/// https://github.com/tock/tock/blob/master/kernel/src/returncode.rs). In
/// particular, a negative value represents an error condition while a
/// nonnegative value represents a success.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ReturnCode {
    value: isize,
}

// This API is largely based off Result's API. Code that wants to match on this
// like a result can convert it to a result using .as_result().
impl ReturnCode {
    /// Creates a new `ReturnCode` using the provided kernel return value.
    pub fn new(value: isize) -> ReturnCode {
        ReturnCode { value }
    }

    /// Converts the `ReturnCode` into a `Result`.
    pub fn as_result(self) -> Result<isize, ErrorCode> {
        if self.value >= 0 {
            Ok(self.value)
        } else {
            Err(ErrorCode::new(self.value))
        }
    }

    /// Returns the value in the form the kernel returned it (an `isize`).
    pub fn value(self) -> isize {
        self.value
    }

    // -------------------------------------------------------------------------
    // Methods below this line are copied directly from Result's interface.
    // -------------------------------------------------------------------------

    /// Returns `true` if this `ReturnCode` represents a success, `false` if it
    /// represents an error.
    pub fn is_ok(self) -> bool {
        self.value >= 0
    }

    /// Returns `true` if this `ReturnCode` represents a failure, `false` if it
    /// represents a success.
    pub fn is_err(self) -> bool {
        self.value < 0
    }

    /// Returns the contained value if this represents a success, and `None` if
    /// it represents an error.
    pub fn ok(self) -> Option<isize> {
        if self.value >= 0 {
            Some(self.value)
        } else {
            None
        }
    }

    /// Returns the error code if this represents an error, and `None` if it
    /// represents a success.
    pub fn err(self) -> Option<ErrorCode> {
        if self.value < 0 {
            Some(ErrorCode::new(self.value))
        } else {
            None
        }
    }

    /// Applies a function to the contained value (if the value is a success),
    /// or returns the provided default (if the value is an error).
    pub fn map_or<U, F: FnOnce(isize) -> U>(self, default: U, f: F) -> U {
        if self.value >= 0 {
            f(self.value)
        } else {
            default
        }
    }

    /// Map a ReturnCode to a U by applying value_f to a success value or
    /// error_f to an error value.
    pub fn map_or_else<U, D: FnOnce(ErrorCode) -> U, F: FnOnce(isize) -> U>(
        self,
        error_f: D,
        value_f: F,
    ) -> U {
        if self.value >= 0 {
            value_f(self.value)
        } else {
            error_f(ErrorCode::new(self.value))
        }
    }
}
