use crate::Register;

/// `RawSyscalls` allows a fake Tock kernel to be injected into components for
/// unit testing. It is implemented by `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::fake::Kernel`. **Components should not use `RawSyscalls`
/// directly; instead, use the `Syscalls` trait, which provides higher-level
/// interfaces to the system calls.**
///
/// # Safety
/// `RawSyscalls` is unsafe because `unsafe` code depends on its methods to
/// return the correct register values.

// The RawSyscalls trait is designed to minimize the complexity and size of its
// implementation, as its implementation is difficult to test (it cannot be used
// in unit tests, with sanitizers, or in Miri). It is also designed to minimize
// the number of unnecessary instructions it generates.
//
// Convention: This file uses the same register naming conventions as the Tock
// 2.0 syscall TRD. Registers r0-r4 correspond to ARM registers r0-r4 and RISC-V
// registers a0-a4.
//
// Theoretically, RawSyscalls could consist of a single raw system call. To
// start, something like this should work:
//
//   unsafe fn syscall<const CLASS: usize>([Reg; 4]) -> [Reg; 4];
//
// Note: Reg is an abbreviation of Register.
//
// Using a single system call has a major inefficiency. The single raw system
// call would need to clobber every register that any system call can clobber.
// Yield has a far longer clobber list than most system calls, so this would be
// inefficient for the majority of system calls. As a result, we can split yield
// out into its own function, giving the following API:
//
//   unsafe fn yield([Reg; 4]) -> [Reg; 4];
//   unsafe fn syscall<const CLASS: usize>([Reg; 4]) -> [Reg; 4];
//
// There is one significant inefficiency remaining. Many system calls, such as
// memop's "get RAM start address" operation, do not need to set all four
// arguments. The compiler cannot optimize away this inefficiency, so to remove
// it we need to split the system calls up based on the number of arguments they
// take:
//
//   unsafe fn yield0([Reg; 0]) -> [Reg; 4];
//   unsafe fn yield1([Reg; 1]) -> [Reg; 4];
//   unsafe fn yield2([Reg; 2]) -> [Reg; 4];
//   unsafe fn yield3([Reg; 3]) -> [Reg; 4];
//   unsafe fn yield4([Reg; 4]) -> [Reg; 4];
//   unsafe fn syscall0<const CLASS: usize>([Reg; 0]) -> [Reg; 4];
//   unsafe fn syscall1<const CLASS: usize>([Reg; 1]) -> [Reg; 4];
//   unsafe fn syscall2<const CLASS: usize>([Reg; 2]) -> [Reg; 4];
//   unsafe fn syscall3<const CLASS: usize>([Reg; 3]) -> [Reg; 4];
//   unsafe fn syscall4<const CLASS: usize>([Reg; 4]) -> [Reg; 4];
//
// However, not all of these are used! If we remove the system calls that are
// unused, we are left with the following:
//
//   unsafe fn yield1([Reg; 1]) -> [Reg; 4];
//   unsafe fn yield2([Reg; 2]) -> [Reg; 4];
//   unsafe fn syscall1<const CLASS: usize>([Reg; 1]) -> [Reg; 4];
//   unsafe fn syscall2<const CLASS: usize>([Reg; 2]) -> [Reg; 4];
//   unsafe fn syscall4<const CLASS: usize>([Reg; 4]) -> [Reg; 4];
//
// These system calls are refined further individually, which is documented on
// a per-function basis.
pub unsafe trait RawSyscalls: Sized {
    // yield1 can only be used to call `yield-wait`, which does not have a
    // return value. To simplify the assembly implementation, we remove its
    // return value.
    //
    // yield1 should:
    //     1. Call syscall class 0
    //     2. Pass in r0 as an inlateout register.
    //     3. Mark all caller-saved registers as lateout clobbers.
    //     4. NOT provide any of the following options:
    //            pure             (yield has side effects)
    //            nomem            (a callback can read + write globals)
    //            readonly         (a callback can write globals)
    //            preserves_flags  (a callback can change flags)
    //            noreturn         (yield is expected to return)
    //            nostack          (a callback needs the stack)
    /// `yield1` should only be called by `libtock_platform`.
    /// # Safety
    /// yield1 may only be used for yield operations that do not return a value.
    /// It is exactly as safe as the underlying system call.
    unsafe fn yield1(_: [Register; 1]);

    // yield2 can only be used to call `yield-no-wait`. `yield-no-wait` does not
    // return any values, so to simplify the assembly we omit return arguments.
    //
    // yield2 should:
    //     1. Call syscall class 0
    //     2. Pass in r0 and r1 as inlateout registers.
    //     3. Mark all caller-saved registers as lateout clobbers.
    //     4. NOT provide any of the following options:
    //            pure             (yield has side effects)
    //            nomem            (a callback can read + write globals)
    //            readonly         (a callback can write globals)
    //            preserves_flags  (a callback can change flags)
    //            noreturn         (yield is expected to return)
    //            nostack          (a callback needs the stack)
    /// `yield2` should only be called by `libtock_platform`.
    /// # Safety
    /// yield2 may only be used for yield operations that do not return a value.
    /// It has the same safety invariants as the underlying system call.
    unsafe fn yield2(_: [Register; 2]);

    // syscall1 is only used to invoke Memop operations. Because there are no
    // Memop commands that set r2 or r3, raw_syscall1 only needs to return r0
    // and r1.
    //
    // Memop commands may panic in the unit test environment, as not all memop
    // calls can be sensibly implemented in that environment.
    //
    // syscall1 should:
    //     1. Call the syscall class specified by CLASS.
    //     2. Pass r0 as an inlateout register.
    //     3. Specify r1 as a lateout register and return its value.
    //     4. Not mark any registers as clobbered.
    //     5. Have all of the following options:
    //            preserves_flags
    //            nostack
    //            nomem            (it is okay for the compiler to cache globals
    //                              across memop calls)
    //     6. NOT have any of the following options:
    //            pure      (two invocations of the same memop can return
    //                       different values)
    //            readonly  (incompatible with nomem)
    //            noreturn
    /// `syscall1` should only be called by `libtock_platform`.
    /// # Safety
    /// This directly makes a system call. It can only be used for Memop calls
    /// that accept 1 argument and only overwrite r0 and r1 on return. It is
    /// unsafe any time the underlying system call is unsafe.
    unsafe fn syscall1<const CLASS: usize>(_: [Register; 1]) -> [Register; 2];

    // syscall2 is used to invoke Exit as well as Memop operations that take an
    // argument. Memop does not currently use more than 2 registers for its
    // return value, and Exit does not return, so syscall2 only returns 2
    // values.
    //
    // syscall2 should:
    //     1. Call the syscall class specified by CLASS.
    //     2. Pass r0 and r1 as inlateout registers.
    //     3. Not mark any registers as clobbered.
    //     4. Have all of the following options:
    //            preserves_flags
    //            nostack
    //            nomem            (the compiler can cache globals across memop
    //                              calls)
    //     5. NOT have any of the following options:
    //            pure      Two invocations of sbrk can return different values
    //            readonly  Incompatible with nomem
    //            noreturn
    /// `syscall2` should only be called by `libtock_platform`.
    /// # Safety
    /// `syscall2` directly makes a system call. It can only be used for core
    /// kernel system calls that accept 2 arguments and only overwrite r0 and r1
    /// on return. It is unsafe any time the underlying system call is unsafe.
    unsafe fn syscall2<const CLASS: usize>(_: [Register; 2]) -> [Register; 2];

    // syscall4 should:
    //     1. Call the syscall class specified by CLASS.
    //     2. Pass r0-r3 in the corresponding registers as inlateout registers.
    //     3. Not mark any registers as clobbered.
    //     4. Have all of the following options:
    //            preserves_flags  (these system calls do not touch flags)
    //            nostack          (these system calls do not touch the stack)
    //     5. NOT have any of the following options:
    //            pure      (these system calls have side effects)
    //            nomem     (the compiler needs to write to globals before allow)
    //            readonly  (rw allow can modify memory)
    //            noreturn  (all these system calls are expected to return)
    //
    // For subscribe(), the callback pointer should be either 0 (for the null
    // callback) or an `unsafe extern fn(u32, u32, u32, Register)`.
    /// `syscall4` should only be called by `libtock_platform`.
    ///
    /// # Safety
    /// `syscall4` must NOT be used to invoke yield. It inherits all safety
    /// invariants from the underlying system call as described in TRD 104,
    /// which varies depending on the system call class.
    ///
    /// For the Allow system calls, there are some invariants that are stricter
    /// than TRD 104. These invariants are explicitly allowed by TRD 104.
    ///
    /// For Read-Only Allow, the aliasing invariants on the buffer are
    /// equivalent to passing a `&[u8]` reference across the system call
    /// boundary. In particular, that means there MUST NOT be a `&mut [u8]`
    /// reference overlapping the passed buffer, until the buffer has been
    /// returned by a Read-Only Allow call.
    ///
    /// For Read-Write Allow, the aliasing invariants on the buffer are
    /// equivalent to passing a `&mut [u8]` reference across the system call
    /// boundary. In particular, that means there MUST NOT be a reference
    /// overlapping the passed buffer, until the buffer has been returned by a
    /// Read-Write Allow call.
    unsafe fn syscall4<const CLASS: usize>(_: [Register; 4]) -> [Register; 4];
}
