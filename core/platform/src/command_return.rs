use crate::{return_type, ErrorCode, ReturnType};

/// The response type from `command`. Can represent a successful value or a
/// failure.
#[derive(Clone, Copy)]
pub struct CommandReturn {
    pub(crate) return_type: ReturnType,
    // r1, r2, and r3 should only contain 32-bit values. However, these are
    // converted directly from usizes returned by RawSyscalls::four_arg_syscall.
    // To avoid casting twice (both when converting to a Command Return and when
    // calling a get_*() function), we store the usizes directly. Then using the
    // CommandReturn only involves one conversion for each of r1, r2, and r3,
    // performed in the get_*() functions.
    pub(crate) r1: usize,
    pub(crate) r2: usize,
    pub(crate) r3: usize,
}

impl CommandReturn {
    // I generally expect CommandReturn to be used with pattern matching, e.g.:
    //
    //     let command_return = Syscalls::command(314, 1, 1, 2);
    //     if let Some((val1, val2)) = command_return.get_success_2_u32() {
    //         // ...
    //     } else if let Some(error_code) = command_return.get_failure() {
    //         // ...
    //     } else {
    //         // Incorrect return type
    //     }

    /// Returns true if this CommandReturn is of type Failure. Note that this
    /// does not return true for other failure types, such as Failure with u32.
    pub fn is_failure(&self) -> bool {
        self.return_type == return_type::FAILURE
    }

    /// Returns true if this CommandReturn is of type Failure with u32.
    pub fn is_failure_u32(&self) -> bool {
        self.return_type == return_type::FAILURE_U32
    }

    /// Returns true if this CommandReturn is of type Failure with 2 u32.
    pub fn is_failure_2_u32(&self) -> bool {
        self.return_type == return_type::FAILURE_2_U32
    }

    /// Returns true if this CommandReturn is of type Failure with u64.
    pub fn is_failure_u64(&self) -> bool {
        self.return_type == return_type::FAILURE_U64
    }

    /// Returns true if this CommandReturn is of type Success. Note that this
    /// does not return true for other success types, such as Success with u32.
    pub fn is_success(&self) -> bool {
        self.return_type == return_type::SUCCESS
    }

    /// Returns true if this CommandReturn is of type Success with u32.
    pub fn is_success_u32(&self) -> bool {
        self.return_type == return_type::SUCCESS_U32
    }

    /// Returns true if this CommandReturn is of type Success with 2 u32.
    pub fn is_success_2_u32(&self) -> bool {
        self.return_type == return_type::SUCCESS_2_U32
    }

    /// Returns true if this CommandReturn is of type Success with u64.
    pub fn is_success_u64(&self) -> bool {
        self.return_type == return_type::SUCCESS_U64
    }

    /// Returns true if this CommandReturn is of type Success with 3 u32.
    pub fn is_success_3_u32(&self) -> bool {
        self.return_type == return_type::SUCCESS_3_U32
    }

    /// Returns true if this CommandReturn is of type Success with u32 and u64.
    pub fn is_success_u32_u64(&self) -> bool {
        self.return_type == return_type::SUCCESS_U32_U64
    }

    /// Returns the error code if this CommandReturn is of type Failure.
    pub fn get_failure(&self) -> Option<ErrorCode> {
        if !self.is_failure() {
            return None;
        }
        Some(self.r1.into())
    }

    /// Returns the error code and value if this CommandReturn is of type
    /// Failure with u32.
    pub fn get_failure_u32(&self) -> Option<(ErrorCode, u32)> {
        if !self.is_failure_u32() {
            return None;
        }
        Some((self.r1.into(), self.r2 as u32))
    }

    /// Returns the error code and return values if this CommandReturn is of
    /// type Failure with 2 u32.
    pub fn get_failure_2_u32(&self) -> Option<(ErrorCode, u32, u32)> {
        if !self.is_failure_2_u32() {
            return None;
        }
        Some((self.r1.into(), self.r2 as u32, self.r3 as u32))
    }

    /// Returns the error code and return value if this CommandReturn is of type
    /// Failure with u64.
    pub fn get_failure_u64(&self) -> Option<(ErrorCode, u64)> {
        if !self.is_failure_u64() {
            return None;
        }
        Some((self.r1.into(), self.r2 as u64 + ((self.r3 as u64) << 32)))
    }

    /// Returns the value if this CommandReturn is of type Success with u32.
    pub fn get_success_u32(&self) -> Option<u32> {
        if !self.is_success_u32() {
            return None;
        }
        Some(self.r1 as u32)
    }

    /// Returns the values if this CommandReturn is of type Success with 2 u32.
    pub fn get_success_2_u32(&self) -> Option<(u32, u32)> {
        if !self.is_success_2_u32() {
            return None;
        }
        Some((self.r1 as u32, self.r2 as u32))
    }

    /// Returns the value if this CommandReturn is of type Success with u64.
    pub fn get_success_u64(&self) -> Option<u64> {
        if !self.is_success_u64() {
            return None;
        }
        Some(self.r1 as u64 + ((self.r2 as u64) << 32))
    }

    /// Returns the values if this CommandReturn is of type Success with 3 u32.
    pub fn get_success_3_u32(&self) -> Option<(u32, u32, u32)> {
        if !self.is_success_3_u32() {
            return None;
        }
        Some((self.r1 as u32, self.r2 as u32, self.r3 as u32))
    }

    /// Returns the values if this CommandReturn is of type Success with u32 and
    /// u64.
    pub fn get_success_u32_u64(&self) -> Option<(u32, u64)> {
        if !self.is_success_u32_u64() {
            return None;
        }
        Some((self.r1 as u32, self.r2 as u64 + ((self.r3 as u64) << 32)))
    }

    /// Returns the return type of this command.
    pub fn return_type(&self) -> ReturnType {
        self.return_type
    }
}
