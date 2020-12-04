// TODO: Implement `libtock_runtime` and `libtock_unittest`, which are
// referenced in the comment on `RawSyscalls`.

/// `RawSyscalls` allows a fake Tock kernel to be injected into components for
/// unit testing. It is implemented by `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::FakeSyscalls`. Components should not use `RawSyscalls`
/// directly; instead, use the `Syscalls` trait, which provides higher-level
/// interfaces to the system calls.

// RawSyscalls is designed to minimize the amount of handwritten assembly code
// needed without generating unnecessary instructions. There are a few major
// factors affecting its design:
//     1. Most system calls only clobber r0-r4 (*), while yield has a far longer
//        clobber list. As such, yield must have its own assembly
//        implementation.
//     2. The compiler is unable to optimize away unused arguments. For example,
//        memop's "get process RAM start address" operation only needs r0 set,
//        while memop's "break" operation needs both r0 and r1 set. If our
//        inline assembly calls "get process RAM start address" but sets both r0
//        and r1, the compiler doesn't know that r1 will be
//        ignored so setting that register will not be optimized away. Therefore
//        we want to set the minimum number of argument registers possible.
//     3. The cost of specifying unused return registers is only that of
//        unnecessarily marking a register as clobbered. Explanation: After
//        inlining, an unused register is marked as "changed by the assembly"
//        but can immediately be re-used by the compiler, which is the same as a
//        clobbered register. System calls should generally be
//        inlined -- and even if they aren't, the unused return values will
//        probably be passed in caller-saved registers (this is true for the C
//        ABI, so probably true for the Rust ABI), which are treated as
//        clobbered regardless.
//
// (*) When this file refers to registers, it uses the same naming convention as
// the Tock 2.0 syscalls TRD. Registers r0-r4 correspond to ARM registers r0-r4
// and RISC-V registers a0-a4.
//
// Currently, yield takes exactly one argument, to specify what yield type to
// do. Therefore we only need one raw yield call.
//
// Subscribe, command, read-write allow, and read-only allow all take four
// argument types. Even when calling command IDs that have unused arguments, we
// still need to clear the argument registers so as to avoid passing
// confidential data to capsules (this is in line with Tock's threat model). As
// such, four_arg_syscall() is used for all subscribe, command, read-only allow,
// and read-write allow system calls.
//
// Memop takes 1 or 2 arguments (operation and an optional argument). Because it
// is part of the core kernel, it is okay for us to leave arbitrary data in the
// argument register for operations where the argument register is unused
// (again, in line with Tock's threat model). As such, for efficiency, we need
// two raw memop calls: one for operations without an argument and one for
// operations with an argument.
//
// The success type for Memop calls depends on the operation perform. However,
// all *currently defined* memop operations return either Success or Success
// with u32. Therefore, the memop implementations only need to mark r0 and r1 as
// clobbered, not r2 and r3. This choice of clobbers will need to be revisited
// if and when a memop operation that returns more data is added.
//
// The decision of where to use u32 and usize can be a bit tricky. The Tock
// syscall ABI is currently only specified for 32-bit systems, so on real Tock
// systems both types match the size of a register, but the unit test
// environment can be either 32 bit or 64 bit. This interface uses usize for
// values that can contain pointers, so that pointers are not truncated in the
// unit test environment. To keep types as consistent as possible, it uses u32
// for all values that cannot be pointers.
pub trait RawSyscalls {
    // raw_yield should:
    //     1. Call syscall class 0
    //     2. Use register r0 for input and output as an inlateout register,
    //        passing in r0_in and returning its value.
    //     3. Mark all caller-saved registers as lateout clobbers.
    //     4. NOT provide any of the following options:
    //            pure             (yield has side effects)
    //            nomem            (a callback can read + write globals)
    //            readonly         (a callback can write globals)
    //            preserves_flags  (a callback can change flags)
    //            noreturn         (yield is expected to return)
    //            nostack          (a callback needs the stack)
    //
    // Design note: This is safe because the yield types that currently exist
    // are safe. If an unsafe yield type is added, we will need to make
    // raw_yield unsafe. Although raw_yield shouldn't be called by code outside
    // this crate, it can be, so that is a backwards-incompatible change. We
    // pass YieldType rather than a usize because if we used usize directly then
    // this API becomes unsound if the kernel adds support for an unsafe yield
    // type (or even one that takes one more argument).
    fn raw_yield(r0_in: YieldType) -> u32;

    // four_arg_syscall is used to invoke the subscribe, command, read-write
    // allow, and read-only allow system calls.
    //
    // four_arg_syscall's inline assembly should have the following properties:
    //     1. Calls the syscall class specified by class
    //     2. Passes r0-r3 in the corresponding registers as inlateout
    //        registers. Returns r0-r3 in order.
    //     3. Does not mark any registers as clobbered.
    //     4. Has all of the following options:
    //            preserves_flags  (these system calls do not touch flags)
    //            nostack          (these system calls do not touch the stack)
    //     5. Does NOT have any of the following options:
    //            pure      (these system calls have side effects)
    //            nomem     (the compiler needs to write to globals before allow)
    //            readonly  (rw allow can modify memory)
    //            noreturn  (all these system calls are expected to return)
    //
    // Note that subscribe's application data argument can potentially contain a
    // pointer, so r3 can contain a pointer (in addition to r1 and r2, which
    // more obviously contain pointers for subscribe and memop).
    //
    // For subscribe(), the callback pointer should be either 0 (for the null
    // callback) or an `unsafe extern fn(u32, u32, u32, usize)`.
    /// # Safety
    /// `four_arg_syscall` must NOT be used to invoke yield. Otherwise, it has
    /// the same safety invariants as the underlying system call, which varies
    /// depending on the system call class.
    unsafe fn four_arg_syscall(
        r0: u32,
        r1: usize,
        r2: usize,
        r3: usize,
        class: u8,
    ) -> (u32, usize, usize, usize);

    // zero_arg_memop is used to invoke memop operations that do not accept an
    // argument register. Because there are no memop commands that set r2 or r3,
    // this only needs to return r0 and r1.
    //
    // Memop commands may panic in the unit test environment, as not all memop
    // calls can be sensibly implemented in that environment.
    //
    // zero_arg_memop's inline assembly should have the following properties:
    //     1. Calls syscall class 5
    //     2. Specifies r0 as an inlateout register, and r1 as a lateout
    //        register.
    //     3. Does not mark any registers as clobbered.
    //     4. Has all of the following options:
    //            preserves_flags
    //            nostack
    //            nomem            (it is okay for the compiler to cache globals
    //                              across memop calls)
    //     5. Does NOT have any of the following options:
    //            pure      (two invocations of the same memop can return
    //                       different values)
    //            readonly  (incompatible with nomem)
    //            noreturn
    //
    // Design note: like raw_yield, this is safe because memops that currently
    // exist are safe. zero_arg_memop takes a ZeroArgMemop rather than a u32 so
    // that if the kernel adds an unsafe memop -- or one that can clobber r2/r3
    // --  this API doesn't become unsound.
    fn zero_arg_memop(r0_in: ZeroArgMemop) -> (u32, usize);

    // one_arg_memop is used to invoke memop operations that take an argument.
    // Because there are no memop operations that set r2 or r3, this only needs
    // to return r0 and r1.
    //
    // one_arg_memop's inline assembly should:
    //     1. Call syscall class 5
    //     2. Specify r0 and r1 as inlateout registers, and return (r0, r1)
    //     3. Not mark any registers as clobbered.
    //     4. Have all of the following options:
    //            preserves_flags
    //            nostack
    //            nomem            (the compiler can cache globals across memop
    //                              calls)
    //     5. Does NOT have any of the following options:
    //            pure      Two invocations of sbrk can return different values
    //            readonly  Incompatible with nomem
    //            noreturn
    //
    // Design note: like raw_yield, this is safe because memops that currently
    // exist are safe. zero_arg_memop takes a ZeroArgMemop rather than a u32 so
    // that if the kernel adds an unsafe memop -- or one that can clobber r2/r3
    // -- this API doesn't become unsound.
    fn one_arg_memop(r0_in: OneArgMemop, r1: usize) -> (u32, usize);
}

#[non_exhaustive]
#[repr(u32)]
pub enum OneArgMemop {
    Brk = 0,
    Sbrk = 1,
    FlashRegionStart = 8,
    FlashRegionEnd = 9,
    SpecifyStackTop = 10,
    SpecifyHeapStart = 11,
    // Note: before adding new memop operations, make sure the assumptions in
    // the design notes on `one_arg_memop` are valid for the new operation type.
}

// TODO: When the numeric values (0 and 1) are assigned to the yield types,
// specify those values here.
#[non_exhaustive]
#[repr(u32)]
pub enum YieldType {
    Wait,
    NoWait,
}

#[non_exhaustive]
#[repr(u32)]
pub enum ZeroArgMemop {
    MemoryStart = 2,
    MemoryEnd = 3,
    FlashStart = 4,
    FlashEnd = 5,
    GrantStart = 6,
    FlashRegions = 7,
    // Note: before adding new memop operations, make sure the assumptions in
    // the design notes on `zero_arg_memop` are valid for the new operation
    // type.
}
