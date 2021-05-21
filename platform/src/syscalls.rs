/// `Syscalls` provides safe abstractions over Tock's system calls. It is
/// implemented for `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::fake::Kernel` (by way of `RawSyscalls`).
pub trait Syscalls {
    /// Runs the next pending callback, if a callback is pending. Unlike
    /// `yield_wait`, `yield_no_wait` returns immediately if no callback is
    /// pending.
    fn yield_no_wait() -> crate::YieldNoWaitReturn;

    /// Puts the process to sleep until a callback becomes pending, invokes the
    /// callback, then returns.
    fn yield_wait();

    // TODO: Add a subscribe interface.

    // TODO: Add a command interface.

    // TODO: Add a read-write allow interface.

    // TODO: Add a read-only allow interface.

    // TODO: Add memop() methods.
}
