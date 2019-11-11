//! Interface to the Low Level Debug capsule. This provides routines that are
//! useful for toolchain issues. The capsule is documented at:
//! https://github.com/tock/tock/blob/master/doc/syscalls/00008_low_level_debug.md

use crate::syscalls::{command, command1_insecure};

const DRIVER_NUMBER: usize = 8;

mod command_nr {
    pub const ALERT_CODE: usize = 1;
    pub const PRINT1: usize = 2;
    pub const PRINT2: usize = 3;
}

/// Use the LowLevelDebug capsule (if present) to indicate the given status
/// code. If the capsule is not present, this is a no-op.
#[inline(always)] // Improve reliability for relocation issues
pub fn low_level_status_code(code: usize) {
    unsafe {
        command1_insecure(DRIVER_NUMBER, command_nr::ALERT_CODE, code);
    }
}

/// Use the LowLevelDebug capsule (if present) to print a single number. If the
/// capsule is not present, this is a no-op.
#[inline(always)] // Improve reliability for relocation issues
pub fn low_level_print1(value: usize) {
    unsafe {
        command1_insecure(DRIVER_NUMBER, command_nr::PRINT1, value);
    }
}

/// Use the LowLevelDebug capsule (if present) to print two numbers. If the
/// capsule is not present, this is a no-op.
#[inline(always)] // Improve reliability for relocation issues
pub fn low_level_print2(value1: usize, value2: usize) {
    unsafe {
        command(DRIVER_NUMBER, command_nr::PRINT2, value1, value2);
    }
}
