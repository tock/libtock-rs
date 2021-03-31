// TODO: Implement `libtock_unittest`, which is referenced in the comment on
// `RawSyscalls`.

/// `RawSyscalls` allows a fake Tock kernel to be injected into components for
/// unit testing. It is implemented by `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::fake::Kernel`. **Components should not use `RawSyscalls`
/// directly; instead, use the `Syscalls` trait, which provides higher-level
/// interfaces to the system calls.**

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
//   unsafe fn syscall<const CLASS: usize>([usize; 4]) -> [usize; 4];
//
// However, this will not work with Miri's -Zmiri-track-raw-pointers flag, as it
// causes pointers passed to the kernel via the Allow system calls to be
// untagged. In order to work with -Zmiri-track-raw-pointers, we need to pass
// pointers for the register values. Rust's closest analogue to C's void pointer
// is *mut () or *const (); we use *mut () because it is shorter:
//
//   unsafe fn syscall<const CLASS: usize>([*mut (); 4]) -> [*mut (); 4];
//
// Using a single system call has a major inefficiency. The single raw system
// call would need to clobber every register that any system call can clobber.
// Yield has a far longer clobber list than most system calls, so this would be
// inefficient for the majority of system calls. As a result, we can split yield
// out into its own function, giving the following API:
//
//   unsafe fn yield([*mut (); 4]) -> [*mut (); 4];
//   unsafe fn syscall<const CLASS: usize>([*mut (); 4]) -> [*mut (); 4];
//
// There is one significant inefficiency remaining. Many system calls, such as
// memop's "get RAM start address" operation, do not need to set all four
// arguments. The compiler cannot optimize away this inefficiency, so to remove
// it we need to split the system calls up based on the number of arguments they
// take:
//
//   unsafe fn yield0([*mut (); 0]) -> [*mut (); 4];
//   unsafe fn yield1([*mut (); 1]) -> [*mut (); 4];
//   unsafe fn yield2([*mut (); 2]) -> [*mut (); 4];
//   unsafe fn yield3([*mut (); 3]) -> [*mut (); 4];
//   unsafe fn yield4([*mut (); 4]) -> [*mut (); 4];
//   unsafe fn syscall0<const CLASS: usize>([*mut (); 0]) -> [*mut (); 4];
//   unsafe fn syscall1<const CLASS: usize>([*mut (); 1]) -> [*mut (); 4];
//   unsafe fn syscall2<const CLASS: usize>([*mut (); 2]) -> [*mut (); 4];
//   unsafe fn syscall3<const CLASS: usize>([*mut (); 3]) -> [*mut (); 4];
//   unsafe fn syscall4<const CLASS: usize>([*mut (); 4]) -> [*mut (); 4];
//
// However, not all of these are used! If we remove the system calls that are
// unused, we are left with the following:
//
//   unsafe fn yield1([*mut (); 1]) -> [*mut (); 4];
//   unsafe fn yield2([*mut (); 2]) -> [*mut (); 4];
//   unsafe fn syscall1<const CLASS: usize>([*mut (); 1]) -> [*mut (); 4];
//   unsafe fn syscall2<const CLASS: usize>([*mut (); 2]) -> [*mut (); 4];
//   unsafe fn syscall4<const CLASS: usize>([*mut (); 4]) -> [*mut (); 4];
//
// To avoid making the RawSyscalls implementation index into arrays, we replace
// the arrays in the input with multiple arguments. For symmetry, we also
// replace the output with a tuple of individual values. This gives:
//
//   unsafe fn yield1(*mut ()) -> (*mut (), *mut (), *mut (), *mut ());
//
//   unsafe fn yield2(*mut (), *mut ()) -> (*mut (), *mut (), *mut (), *mut ());
//
//   unsafe fn syscall1<const CLASS: usize>(*mut ())
//       -> (*mut (), *mut (), *mut (), *mut ());
//
//   unsafe fn syscall2<const CLASS: usize>(*mut (), *mut ())
//       -> (*mut (), *mut (), *mut (), *mut ());
//
//   unsafe fn syscall4<const CLASS: usize>(*mut (), *mut (), *mut (), *mut ())
//       -> (*mut (), *mut (), *mut (), *mut ());
//
// These system calls are refined further individually, which is documented on
// a per-function basis.
pub unsafe trait RawSyscalls {
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
    unsafe fn yield1(r0: *mut ());

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
    unsafe fn yield2(r0: *mut (), r1: *mut ());

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
    /// This directly makes a system call. It can only be used for core kernel
    /// system calls that accept 1 argument and only overwrite r0 and r1 on
    /// return. It is unsafe any time the underlying system call is unsafe.
    unsafe fn syscall1<const CLASS: usize>(r0: *mut ()) -> (*mut (), *mut ());

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
    unsafe fn syscall2<const CLASS: usize>(r0: *mut (), r1: *mut ()) -> (*mut (), *mut ());

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
    // callback) or an `unsafe extern fn(u32, u32, u32, Userdata)`.
    /// `syscall4` should only be called by `libtock_platform`.
    ///
    /// # Safety
    /// `syscall4` must NOT be used to invoke yield. Otherwise, it has the same
    /// safety invariants as the underlying system call, which varies depending
    /// on the system call class.
    unsafe fn syscall4<const CLASS: usize>(
        r0: *mut (),
        r1: *mut (),
        r2: *mut (),
        r3: *mut (),
    ) -> (*mut (), *mut (), *mut (), *mut ());
}
