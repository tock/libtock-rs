use crate::{return_variant, ErrorCode, ReturnVariant};

use core::mem::transmute;

/// The response type from the [`command`](crate::Syscalls::command) syscall.
/// Can represent a success or a failure with or without associated data.
///
/// After a syscall is made, registers `r1`-`r3` contain the output as
/// described by [TRD 104][trd-104]. Some syscalls only return success/failure,
/// while others provide associated data. This is done by placing the _return
/// variant_ in `r0`, which specifies how the output registers should be
/// interpreted. For syscalls other than `command`, the possible output
/// variants are fixed; you always know which variants are expected given the
/// syscall class.
///
/// However, the `command` syscall is flexible - there must be one success
/// variant and one failure variant for a given driver/command ID, but
/// which variants those are, and what data is expected, cannot be checked
/// statically. Capsules and userspace APIs must agree on the expected
/// variants for success and failure.
///
/// # Example
///
/// This uses the `get_*` methods to check the variants explicitly and extract
/// the associated data.
///
/// ```no_run
/// let command_return = Syscalls::command(314, 1, 1, 2);
/// if let Some((val1, val2)) = command_return.get_success_2_u32() {
///     // If there was a success, there is an associated data (u32, u32).
/// } else if let Some(error_code) = command_return.get_failure() {
///     // If there was a failure, there's no associated data and we only
///     // have an error code.
/// } else {
///     // Incorrect return variant! If this occurs, your capsule and userspace
///     // API do not agree on what the return variants should be.
///     // An application may want to panic in this case to catch this early.
/// }
/// ```
///
/// [trd-104]: https://github.com/tock/tock/blob/master/doc/reference/trd104-syscalls.md#32-return-values
#[must_use = "this `CommandReturn` may represent an error, which should be handled"]
#[derive(Clone, Copy, Debug)]
pub struct CommandReturn {
    return_variant: ReturnVariant,

    // Safety invariant on r1: If return_variant is failure variant, r1 must be
    // a valid ErrorCode.
    r1: u32,
    r2: u32,
    r3: u32,
}

impl CommandReturn {
    /// # Safety
    /// If return_variant is a failure variant, r1 must be a valid ErrorCode.
    pub unsafe fn new(return_variant: ReturnVariant, r1: u32, r2: u32, r3: u32) -> Self {
        CommandReturn {
            return_variant,
            r1,
            r2,
            r3,
        }
    }

    /// Returns true if this CommandReturn is of type Failure. Note that this
    /// does not return true for other failure types, such as Failure with u32.
    pub fn is_failure(&self) -> bool {
        self.return_variant == return_variant::FAILURE
    }

    /// Returns true if this CommandReturn is of type Failure with u32.
    pub fn is_failure_u32(&self) -> bool {
        self.return_variant == return_variant::FAILURE_U32
    }

    /// Returns true if this CommandReturn is of type Failure with 2 u32.
    pub fn is_failure_2_u32(&self) -> bool {
        self.return_variant == return_variant::FAILURE_2_U32
    }

    /// Returns true if this CommandReturn is of type Failure with u64.
    pub fn is_failure_u64(&self) -> bool {
        self.return_variant == return_variant::FAILURE_U64
    }

    /// Returns true if this CommandReturn is of type Success. Note that this
    /// does not return true for other success types, such as Success with u32.
    pub fn is_success(&self) -> bool {
        self.return_variant == return_variant::SUCCESS
    }

    /// Returns true if this CommandReturn is of type Success with u32.
    pub fn is_success_u32(&self) -> bool {
        self.return_variant == return_variant::SUCCESS_U32
    }

    /// Returns true if this CommandReturn is of type Success with 2 u32.
    pub fn is_success_2_u32(&self) -> bool {
        self.return_variant == return_variant::SUCCESS_2_U32
    }

    /// Returns true if this CommandReturn is of type Success with u64.
    pub fn is_success_u64(&self) -> bool {
        self.return_variant == return_variant::SUCCESS_U64
    }

    /// Returns true if this CommandReturn is of type Success with 3 u32.
    pub fn is_success_3_u32(&self) -> bool {
        self.return_variant == return_variant::SUCCESS_3_U32
    }

    /// Returns true if this CommandReturn is of type Success with u32 and u64.
    pub fn is_success_u32_u64(&self) -> bool {
        self.return_variant == return_variant::SUCCESS_U32_U64
    }

    /// Returns the error code if this CommandReturn is of type Failure.
    pub fn get_failure(&self) -> Option<ErrorCode> {
        if !self.is_failure() {
            return None;
        }
        Some(unsafe { transmute(self.r1 as u16) })
    }

    /// Returns the error code and value if this CommandReturn is of type
    /// Failure with u32.
    pub fn get_failure_u32(&self) -> Option<(ErrorCode, u32)> {
        if !self.is_failure_u32() {
            return None;
        }
        Some((unsafe { transmute(self.r1 as u16) }, self.r2))
    }

    /// Returns the error code and return values if this CommandReturn is of
    /// type Failure with 2 u32.
    pub fn get_failure_2_u32(&self) -> Option<(ErrorCode, u32, u32)> {
        if !self.is_failure_2_u32() {
            return None;
        }
        Some((unsafe { transmute(self.r1 as u16) }, self.r2, self.r3))
    }

    /// Returns the error code and return value if this CommandReturn is of type
    /// Failure with u64.
    pub fn get_failure_u64(&self) -> Option<(ErrorCode, u64)> {
        if !self.is_failure_u64() {
            return None;
        }
        Some((
            unsafe { transmute(self.r1 as u16) },
            self.r2 as u64 + ((self.r3 as u64) << 32),
        ))
    }

    /// Returns the value if this CommandReturn is of type Success with u32.
    pub fn get_success_u32(&self) -> Option<u32> {
        if !self.is_success_u32() {
            return None;
        }
        Some(self.r1)
    }

    /// Returns the values if this CommandReturn is of type Success with 2 u32.
    pub fn get_success_2_u32(&self) -> Option<(u32, u32)> {
        if !self.is_success_2_u32() {
            return None;
        }
        Some((self.r1, self.r2))
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
        Some((self.r1, self.r2, self.r3))
    }

    /// Returns the values if this CommandReturn is of type Success with u32 and
    /// u64.
    pub fn get_success_u32_u64(&self) -> Option<(u32, u64)> {
        if !self.is_success_u32_u64() {
            return None;
        }
        Some((self.r1, self.r2 as u64 + ((self.r3 as u64) << 32)))
    }

    /// Returns the register values used to create this command.
    pub fn raw_values(&self) -> (ReturnVariant, u32, u32, u32) {
        (self.return_variant, self.r1, self.r2, self.r3)
    }

    /// Returns the return variant of this command.
    pub fn return_variant(&self) -> ReturnVariant {
        self.return_variant
    }
}
