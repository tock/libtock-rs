use crate::{CommandReturn, YieldNoWaitReturn};

/// `Syscalls` provides safe abstractions over Tock's system calls. It is
/// implemented for `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::fake::Kernel` (by way of `RawSyscalls`).
pub trait Syscalls {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------

    /// Runs the next pending callback, if a callback is pending. Unlike
    /// `yield_wait`, `yield_no_wait` returns immediately if no callback is
    /// pending.
    fn yield_no_wait() -> YieldNoWaitReturn;

    /// Puts the process to sleep until a callback becomes pending, invokes the
    /// callback, then returns.
    fn yield_wait();

    // TODO: Add a subscribe interface.

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    fn command(driver_id: u32, command_id: u32, argument0: u32, argument1: u32) -> CommandReturn;

    // TODO: Add a read-write allow interface.

    // TODO: Add a read-only allow interface.

    // TODO: Add memop() methods.
}
