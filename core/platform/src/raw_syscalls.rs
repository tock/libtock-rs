// TODO: Implement `libtock_unittest`, which is referenced in the comment on
// `RawSyscalls`.

/// `RawSyscalls` allows a fake Tock kernel to be injected into components for
/// unit testing. It is implemented by `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::fake::Kernel`. **Components should not use `RawSyscalls`
/// directly; instead, use the `Syscalls` trait, which provides higher-level
/// interfaces to the system calls.**
// -----------------------------------------------------------------------------
// Full documentation for everything in this file is in doc/RawSyscallsDesign.md
// at the root of this repository.
// -----------------------------------------------------------------------------
use crate::ReturnVariant;

pub trait RawSyscalls {
    // Calls yield-no-wait, passing `flag` to the kernel. The kernel will set
    // the value that flag points to based on whether a callback was executed.
    /// # Safety
    /// Will write a 0 or 1 into flag's pointee.
    unsafe fn raw_yield_no_wait(flag: *mut u8);

    fn raw_yield_wait();

    // `one_arg_syscall` supports calling Exit and Memop operations that do not
    // need an argument.
    /// # Safety
    /// This is a direct wrapper around a raw system call, and produces
    /// undefined behavior exactly when the underlying system call produces
    /// undefined behavior.
    unsafe fn one_arg_syscall(op: u32, class: u8) -> (ReturnVariant, usize);

    // `two_arg_syscall` supports calling Exit and Memop operations that need an
    // argument.
    /// # Safety
    /// This is a direct wrapper around a raw system call, and produces
    /// undefined behavior exactly when the underlying system call produces
    /// undefined behavior.
    unsafe fn two_arg_syscall(op: u32, r1: usize, class: u8) -> (ReturnVariant, usize);

    // `four_arg_syscall` supports calling the Command, Read-Only Allow,
    // Read-Write Allow, and Subscribe system calls.
    /// # Safety
    /// This is a direct wrapper around a raw system call, and produces
    /// undefined behavior exactly when the underlying system call produces
    /// undefined behavior.
    unsafe fn four_arg_syscall(
        r0: u32,
        r1: u32,
        r2: usize,
        r3: usize,
        class: u8,
    ) -> (ReturnVariant, usize, usize, usize);
}
