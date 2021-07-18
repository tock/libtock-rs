//! Implementations of Yield system calls.

use crate::kernel_data::KERNEL_DATA;
use crate::{ExpectedSyscall, SyscallLogEntry};

/// # Safety
/// It must be valid to write a `libtock_platform::YieldNoWaitReturn` into the
/// value pointed to by `return_ptr`. When `yield_no_wait` returns, the value
/// pointed to by `return_ptr` will be set.
pub(super) unsafe fn yield_no_wait(return_ptr: *mut libtock_platform::YieldNoWaitReturn) {
    let override_return = KERNEL_DATA.with(|refcell| {
        let mut refmut = refcell.borrow_mut();
        let kernel_data = refmut
            .as_mut()
            .expect("yield-no-wait called but no fake::Kernel exists");

        kernel_data.syscall_log.push(SyscallLogEntry::YieldNoWait);

        match kernel_data.expected_syscalls.pop_front() {
            None => None,
            Some(ExpectedSyscall::YieldNoWait { override_return }) => override_return,
            Some(expected_syscall) => expected_syscall.panic_wrong_call("yield-no-wait"),
        }
    });

    // TODO: Add the Driver trait and implement driver support, including
    // upcalls.
    let upcall_ran = libtock_platform::YieldNoWaitReturn::NoUpcall;

    unsafe {
        core::ptr::write(return_ptr, override_return.unwrap_or(upcall_ran));
    }
}

pub(super) fn yield_wait() {
    let skip_upcall = KERNEL_DATA.with(|refcell| {
        let mut refmut = refcell.borrow_mut();
        let kernel_data = refmut
            .as_mut()
            .expect("yield-wait called but no fake::Kernel exists");

        kernel_data.syscall_log.push(SyscallLogEntry::YieldWait);

        match kernel_data.expected_syscalls.pop_front() {
            None => false,
            Some(ExpectedSyscall::YieldWait { skip_upcall }) => skip_upcall,
            Some(expected_syscall) => expected_syscall.panic_wrong_call("yield-wait"),
        }
    });

    if skip_upcall {
        return;
    }

    unimplemented!("TODO: Implement upcalls");
}
