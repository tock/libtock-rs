use crate::{
    subscribe, CommandReturn, ErrorCode, RawSyscalls, Subscribe, Upcall, YieldNoWaitReturn,
};

/// `Syscalls` provides safe abstractions over Tock's system calls. It is
/// implemented for `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::fake::Kernel` (by way of `RawSyscalls`).
pub trait Syscalls: RawSyscalls + Sized {
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

    // -------------------------------------------------------------------------
    // Subscribe
    // -------------------------------------------------------------------------

    /// Registers an upcall with the kernel.
    fn subscribe<
        'scope,
        IDS: subscribe::SupportsId<DRIVER_NUM, SUBSCRIBE_NUM>,
        U: Upcall<IDS>,
        CONFIG: subscribe::Config,
        const DRIVER_NUM: u32,
        const SUBSCRIBE_NUM: u32,
    >(
        subscribe: &Subscribe<'scope, Self, DRIVER_NUM, SUBSCRIBE_NUM>,
        upcall: &'scope U,
    ) -> Result<(), ErrorCode>;

    /// Unregisters the upcall with the given ID. If no upcall is registered
    /// with the given ID, `unsubscribe` does nothing.
    fn unsubscribe(driver_num: u32, subscribe_num: u32);

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    fn command(driver_id: u32, command_id: u32, argument0: u32, argument1: u32) -> CommandReturn;

    // TODO: Add a read-write allow interface.

    // TODO: Add a read-only allow interface.

    // TODO: Add memop() methods.

    // -------------------------------------------------------------------------
    // Exit
    // -------------------------------------------------------------------------

    fn exit_terminate(exit_code: u32) -> !;

    fn exit_restart(exit_code: u32) -> !;
}
