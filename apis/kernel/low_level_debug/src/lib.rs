#![no_std]

use libtock_platform::Syscalls;

/// The low-level debug API provides tools to diagnose userspace issues that
/// make normal debugging workflows (e.g. printing to the console) difficult.
///
/// It allows libraries to print alert codes and apps to print numeric
/// information using only the command system call.
///
/// # Example
/// ```ignore
/// use libtock::LowLevelDebug;
///
/// // Prints 0x45 and the app which called it.
/// LowLevelDebug::print_1(0x45);
/// ```

pub struct LowLevelDebug<S: Syscalls>(S);

impl<S: Syscalls> LowLevelDebug<S> {
    /// Run a check against the low-level debug capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn exists() -> bool {
        S::command(DRIVER_NUM, EXISTS, 0, 0).is_success()
    }

    /// Print one of the predefined alerts in [`AlertCode`].
    #[inline(always)]
    pub fn print_alert_code(code: AlertCode) {
        let _ = S::command(DRIVER_NUM, PRINT_ALERT_CODE, code as u32, 0);
    }

    /// Print a single number. The number will be printed in hexadecimal.
    ///
    /// In general, this should only be added temporarily for debugging and
    /// should not be called by released library code.
    #[inline(always)]
    pub fn print_1(x: u32) {
        let _ = S::command(DRIVER_NUM, PRINT_1, x, 0);
    }

    /// Print two numbers. The numbers will be printed in hexadecimal.
    ///
    /// Like `print_1`, this is intended for temporary debugging and should not
    /// be called by released library code. If you want to print multiple
    /// values, it is often useful to use the first argument to indicate what
    /// value is being printed.
    #[inline(always)]
    pub fn print_2(x: u32, y: u32) {
        let _ = S::command(DRIVER_NUM, PRINT_2, x, y);
    }
}

/// A predefined alert code, for use with [`LowLevelDebug::print_alert_code`].
pub enum AlertCode {
    /// Application panic (e.g. `panic!()` called in Rust code).
    Panic = 0x01,

    /// A statically-linked app was not installed in the correct location in
    /// flash.
    WrongLocation = 0x02,
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x8;

// Command IDs
const EXISTS: u32 = 0;
const PRINT_ALERT_CODE: u32 = 1;
const PRINT_1: u32 = 2;
const PRINT_2: u32 = 3;
