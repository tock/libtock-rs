// TODO: Implement `libtock_runtime` and `libtock_unittest`, which are
// referenced in the comment on `Syscalls`.

/// `Syscalls` provides safe abstractions over Tock's system calls. It is
/// implemented for `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::FakeSyscalls` (by way of `RawSyscalls`).
pub trait Syscalls {
    /// Puts the process to sleep until a callback becomes pending, invokes the
    /// callback, then returns.
    fn yield_wait();

    /// Runs the next pending callback, if a callback is pending. Unlike
    /// `yield_wait`, `yield_no_wait` returns immediately if no callback is
    /// pending. Returns true if a callback was executed, false otherwise.
    fn yield_no_wait() -> bool;

    // TODO: Add a subscribe interface.

    // TODO: Add a command interface.

    // TODO: Add a read-write allow interface.

    // TODO: Add a read-only allow interface.

    // TODO: Add memop() methods.
}
