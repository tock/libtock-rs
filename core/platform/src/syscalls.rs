// TODO: Implement `libtock_runtime` and `libtock_unittest`, which are
// referenced in the comment on `Syscalls`.

use crate::syscall_types::{OneArgMemop, ReturnType, YieldType, ZeroArgMemop};

/// `Syscalls` serves two purposes:
///     High level: It provides safe abstractions over Tock's raw system calls.
///     Low level:  It allows a fake Tock kernel to be injected into components
///                 for unit testing.
///
/// To serve these use cases, `Syscalls` has two API layers (called the
/// high-level API and the low-level API). Components needing access to Tock's
/// system calls should use the high-level API, which is safe and has nice
/// abstractions. The low-level API is implemented by
/// `libtock_runtime::TockSyscalls` and `libtock_unittest::FakeSyscalls`, and
/// provides the raw userspace<->kernel interface.
pub trait Syscalls {
    // -------------------------------------------------------------------------
    // High-level API
    // -------------------------------------------------------------------------

    /// Puts the process to sleep until a callback becomes pending, invokes the
    /// callback, then returns.
    fn yield_wait() {
        Self::raw_yield(YieldType::Wait);
    }

    /// Runs the next pending callback, if a callback is pending. Unlike
    /// `yield_wait`, `yield_no_wait` returns immediately if no callback is
    /// pending. Returns true if a callback was executed, false otherwise.
    fn yield_no_wait() -> bool {
        Self::raw_yield(YieldType::NoWait) != ReturnType::Failure as usize
    }

    // TODO: Implement a subscribe interface.

    // TODO: Implement a command interface.

    // TODO: Implement a read-write allow interface.

    // TODO: Implement a read-only allow interface.

    // TODO: Implement memop() methods.

    // -------------------------------------------------------------------------
    // Low-level API
    // -------------------------------------------------------------------------

    // This API is designed to minimize the amount of handwritten assembly code
    // needed without generating unnecessary instructions. There are a few major
    // factors affecting its design:
    //     1. Most system calls only clobber r0-r4, while yield has a far longer
    //        clobber list. As such, yield must have its own assembly
    //        implementation.
    //     2. The compiler is unable to optimize away unused arguments. For
    //        example, memop's "get process RAM start address" operation only
    //        needs r0 set, while memop's "break" operation needs both r0 and r1
    //        set. If our inline assembly calls "get process RAM start address"
    //        but sets both r0 and r1, the compiler doesn't know that r1 will be
    //        ignored so setting that register will not be optimized away.
    //        Therefore we want to set the minimum number of argument registers
    //        possible.
    //     3. The cost of specifying unused return registers is only that of
    //        unnecessarily marking a register as clobbered. Explanation: After
    //        inlining, an unused register is marked as "changed by the
    //        assembly" but can immediately be re-used by the compiler, which is
    //        the same as a clobbered register. System calls should generally be
    //        inlined -- and even if they aren't, the unused return values will
    //        probably be passed in caller-saved registers (this is true for the
    //        C ABI, so probably true for the Rust ABI), which are treated as
    //        clobbered regardless.
    //
    // Currently, yield takes exactly one argument, to specify what yield type
    // to do. Therefore we only need one raw yield call.
    //
    // Subscribe, command, read-write allow, and read-only allow all take four
    // argument types. Even when calling command IDs that have unused arguments,
    // we still need to clear the argument registers so as to avoid passing
    // confidential data to capsules (this is in line with Tock's threat model).
    // As such, four_arg_syscall() is used for all subscribe, command, read-only
    // allow, and read-write allow system calls.
    //
    // Memop takes 1 or 2 arguments (operation and an optional argument), and
    // being part of the core kernel it is okay for us to leave arbitrary data
    // in the argument register if the argument is unused (again, in line with
    // Tock's threat model). Memop returns up to 2 return arguments, so we don't
    // need to mark r2 and r3 as clobbered. As such, we need two raw memop
    // calls: one for operations without an argument and one for operations with
    // an argument.
    //
    // Because the variables passed in and out of raw system calls represent
    // register values, they are of type usize. In cases where it doesn't make
    // sense to pass a pointer-sized value, libtock_unittest::FakeSyscalls is
    // free to panic if a too-large value is passed.

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
    fn raw_yield(r0_in: YieldType) -> usize;

    // four_arg_syscall is used to invoke subscribe, command, read-write allow,
    // and read-only allow system calls.
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
    /// # Safety
    /// A four_arg_syscall must NOT be used to invoke yield. Otherwise, it is
    /// exactly as safe as the underlying system call, which varies depending on
    /// the system call class.
    unsafe fn four_arg_syscall(
        r0: usize,
        r1: usize,
        r2: usize,
        r3: usize,
        class: u8,
    ) -> (usize, usize, usize, usize);

    // zero_arg_memop is used to invoke memop operations that do not accept an
    // argument register. Because the are no memop commands that set r2 or r3,
    // this only needs to return r0 and r1.
    //
    // Many memop commands are not expected to work in the unit test
    // environment. If called, those commands may panic.
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
    // exist are safe. zero_arg_memop takes a ZeroArgMemop rather than a usize
    // so that if the kernel adds an unsafe memop this API doesn't become
    // unsound.
    fn zero_arg_memop(r0_in: ZeroArgMemop) -> (usize, usize);

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
    // exist are safe. zero_arg_memop takes a ZeroArgMemop rather than a usize
    // so that if the kernel adds an unsafe memop this API doesn't become
    // unsound.
    fn one_arg_memop(r0_in: OneArgMemop, r1: usize) -> (usize, usize);
}
