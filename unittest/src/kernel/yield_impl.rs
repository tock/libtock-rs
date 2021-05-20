//! Implementations of Yield system calls.

use crate::kernel::thread_local::get_kernel;
use crate::{ExpectedSyscall, SyscallLogEntry};

/// # Safety
/// It must be valid to write a `libtock_platform::YieldNoWaitReturn` into the
/// value pointed to by `return_ptr`. When `yield_no_wait` returns, the value
/// pointed to by `return_ptr` will be set.
pub(super) unsafe fn yield_no_wait(return_ptr: *mut libtock_platform::YieldNoWaitReturn) {
    let kernel = get_kernel().expect("yield-no-wait called but no fake::Kernel exists");
    kernel.log_syscall(SyscallLogEntry::YieldNoWait);
    let override_return = match kernel.pop_expected_syscall() {
        None => None,
        Some(ExpectedSyscall::YieldNoWait { override_return }) => override_return,
        Some(expected_syscall) => expected_syscall.panic_wrong_call("yield-no-wait"),
    };

    // TODO: Add the Driver trait and implement driver support, including
    // upcalls.
    let upcall_ran = libtock_platform::YieldNoWaitReturn::NoUpcall;

    unsafe {
        core::ptr::write(return_ptr, override_return.unwrap_or(upcall_ran));
    }
}

pub(super) fn yield_wait() {
    let kernel = get_kernel().expect("yield-wait called but no fake::Kernel exists");
    kernel.log_syscall(SyscallLogEntry::YieldWait);
    let skip_upcall = match kernel.pop_expected_syscall() {
        None => false,
        Some(ExpectedSyscall::YieldWait { skip_upcall }) => skip_upcall,
        Some(expected_syscall) => expected_syscall.panic_wrong_call("yield-wait"),
    };
    if skip_upcall {
        return;
    }

    unimplemented!("TODO: Implement upcalls");
}
