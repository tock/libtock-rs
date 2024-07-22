use crate::{
    allow_ro, allow_rw, share, subscribe, AllowRo, AllowRw, CommandReturn, ErrorCode, RawSyscalls,
    Subscribe, Upcall, YieldNoWaitReturn,
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
        'share,
        IDS: subscribe::SupportsId<DRIVER_NUM, SUBSCRIBE_NUM>,
        U: Upcall<IDS>,
        CONFIG: subscribe::Config,
        const DRIVER_NUM: u32,
        const SUBSCRIBE_NUM: u32,
    >(
        subscribe: share::Handle<Subscribe<'share, Self, DRIVER_NUM, SUBSCRIBE_NUM>>,
        upcall: &'share U,
    ) -> Result<(), ErrorCode>;

    /// Unregisters the upcall with the given ID. If no upcall is registered
    /// with the given ID, `unsubscribe` does nothing.
    fn unsubscribe(driver_num: u32, subscribe_num: u32);

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------

    fn command(driver_id: u32, command_id: u32, argument0: u32, argument1: u32) -> CommandReturn;

    // -------------------------------------------------------------------------
    // Read-Write Allow
    // -------------------------------------------------------------------------

    /// Shares a read-write buffer with the kernel.
    fn allow_rw<'share, CONFIG: allow_rw::Config, const DRIVER_NUM: u32, const BUFFER_NUM: u32>(
        allow_rw: share::Handle<AllowRw<'share, Self, DRIVER_NUM, BUFFER_NUM>>,
        buffer: &'share mut [u8],
    ) -> Result<(), ErrorCode>;

    /// Revokes the kernel's access to the buffer with the given ID, overwriting
    /// it with a zero buffer. If no buffer is shared with the given ID,
    /// `unallow_rw` does nothing.
    fn unallow_rw(driver_num: u32, buffer_num: u32);

    // -------------------------------------------------------------------------
    // Read-Only Allow
    // -------------------------------------------------------------------------

    /// Shares a read-only buffer with the kernel.
    fn allow_ro<'share, CONFIG: allow_ro::Config, const DRIVER_NUM: u32, const BUFFER_NUM: u32>(
        allow_ro: share::Handle<AllowRo<'share, Self, DRIVER_NUM, BUFFER_NUM>>,
        buffer: &'share [u8],
    ) -> Result<(), ErrorCode>;

    /// Revokes the kernel's access to the buffer with the given ID, overwriting
    /// it with a zero buffer. If no buffer is shared with the given ID,
    /// `unallow_ro` does nothing.
    fn unallow_ro(driver_num: u32, buffer_num: u32);

    // -------------------------------------------------------------------------
    // Memop
    // -------------------------------------------------------------------------

    /// Changes the location of the program break to the specified address and
    /// returns an error if it fails to do so.
    ///
    /// # Safety
    /// Callers of this function must ensure that they do not pass an
    /// address below any address that includes a currently reachable object.
    unsafe fn memop_brk(addr: *const u8) -> Result<(), ErrorCode>;

    /// Changes the location of the program break by the passed increment,
    /// and returns the previous break address.
    ///
    /// # Safety
    /// Callers of this function must ensure that they do not pass an
    /// increment that would deallocate memory containing any currently
    /// reachable object.
    unsafe fn memop_sbrk(incr: i32) -> Result<*const u8, ErrorCode>;

    /// Increments the program break by the passed increment,
    /// and returns the previous break address.
    fn memop_increment_brk(incr: u32) -> Result<*const u8, ErrorCode>;

    /// Gets the address of the start of this application's RAM allocation.
    fn memop_app_ram_start() -> Result<*const u8, ErrorCode>;

    /// Tells the kernel where the start of the app stack is, to support
    /// debugging.
    fn memop_debug_stack_start(stack_top: *const u8) -> Result<(), ErrorCode>;

    /// Tells the kernel the initial program break, to support debugging.
    fn memop_debug_heap_start(initial_break: *const u8) -> Result<(), ErrorCode>;

    // TODO: Add remaining memop() methods (3-9).

    // -------------------------------------------------------------------------
    // Exit
    // -------------------------------------------------------------------------

    fn exit_terminate(exit_code: u32) -> !;

    fn exit_restart(exit_code: u32) -> !;
}
