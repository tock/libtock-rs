//! Provides the Syscalls trait which directly represents Tock's system call
//! APIs. Syscalls is implemented by both `libtock_runtime` which makes system
//! calls into a real Tock kernel, and `libtock_fake` which is a fake Tock
//! kernel.

// TODO: Implement `libtock_runtime` and `libtock_fake`.

/// Syscalls represents Tock's system call APIs. It is designed to be
/// implemented as easily as possible -- its arguments and return values
/// correspond directly to registers in the ABI. For a higher-level abstraction,
/// see Platform.
///
/// By design, syscalls is designed to be zero-cost in a TBF binary and
/// functional (but not zero-cost) in unit tests. In a TBF binary, Syscalls is
/// implemented with the `'static` lifetime, and is a zero-sized type. Syscalls
/// requires `Copy` in order to support defining it usefully on zero-sized
/// types. When used in unit tests, the Syscalls implementation carries a
/// lifetime local to that unit test.
///
/// With the exception of `memop`, this trait aligns closely to Tock's
/// kernel::Driver trait.
pub trait Syscalls<'k>: Copy {
    /// Calls the `allow` system call.
    ///
    /// # Safety
    /// `allow` is unsafe because callers must guarantee that `pointer` and
    /// `length` refer to memory that the kernel can mutate safely. The buffer
    /// must last for the lifetime 'k.
    // `driver` and `minor` are `usize` because the kernel internally treats
    // them as `usize`s. `allow`'s return value is a kernel `ReturnCode`;
    // Platform translates the `isize` into a `ReturnCode`.
    unsafe fn allow(self, driver: usize, minor: usize, pointer: *mut u8, length: usize) -> isize;

    /// Calls the `command` system call.
    // `driver`, `minor`, `arg1`, and `arg2` are all `usize` (rather than `u32`)
    // because the kernel refers to them internally as `usize`s. command returns
    // a kernel ReturnCode; Platform is responsible for translating an isize
    // into the local ReturnCode.
    fn command(self, driver: usize, minor: usize, arg1: usize, arg2: usize) -> isize;

    /// Calls the `memop` system call with an argument. Note that memop() cannot
    /// cause memory unsafety, although it can cause the app to fault (e.g. Brk
    /// can move the app break below the stack, causing a fault). The isize
    /// returned is a kernel ReturnCode.
    // Platform performs the translation from isize into ReturnCode to keep
    // Syscalls implementations simple.
    fn memop_arg(self, op: MemopWithArg, arg: usize) -> isize;

    /// Calls the `memop` system call with no arguments. This version is
    /// slightly cheaper because it does not need to set the argument register.
    // We're okay with leaking the value in the argument register because
    // memop() is always handled by the core kernel, never by an untrusted
    // capsule.
    fn memop_noarg(self, op: MemopNoArg) -> isize;

    /// Calls the `subscribe` system call.
    ///
    /// # Safety
    /// `subscribe` is unsafe because the callback can potentially be unsafe,
    /// and callers of `subscribe` must assert that calling the callback with
    /// the provided `data` value is safe. The callback must last for the 'k
    /// lifetime.
    // Driver, minor, the callback args, and data are all represented as `usize`
    // because that is the type the kernel uses internally to store them (e.g.
    // as opposed to u32).
    unsafe fn subscribe(
        self,
        driver: usize,
        minor: usize,
        callback: Option<unsafe extern "C" fn(usize, usize, usize, usize)>,
        data: usize,
    );

    /// Puts the process to sleep until a callback becomes pending, then invokes
    /// the callback.
    fn yieldk(self);
}

#[non_exhaustive]
#[repr(usize)]
pub enum MemopWithArg {
    Brk = 0,
    Sbrk = 1,
    FlashRegionStart = 8,
    FlashRegionEnd = 9,
    SpecifyStackTop = 10,
    SpecifyHeapStart = 11,
}

#[non_exhaustive]
#[repr(usize)]
pub enum MemopNoArg {
    MemoryStart = 2,
    MemoryEnd = 3,
    FlashStart = 4,
    FlashEnd = 5,
    GrantStart = 6,
    FlashRegions = 7,
}
