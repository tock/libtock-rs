//! Implementations of Yield system calls.

use crate::kernel_data::{with_kernel_data, KERNEL_DATA};
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

    let upcall_ran = match invoke_next_upcall() {
        true => libtock_platform::YieldNoWaitReturn::Upcall,
        false => libtock_platform::YieldNoWaitReturn::NoUpcall,
    };

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

    // In a real Tock system, a process that calls yield-wait with no queued
    // upcalls would be put to sleep until an upcall was queued (e.g. by an
    // interrupt). However, in this single-threaded test environment, there is
    // no possibility a new upcall will be enqueued while we wait. Panicing is
    // friendlier than hanging, so we panic if there's no upcall.
    assert!(
        invoke_next_upcall(),
        "yield-wait called with no queued upcall"
    );
}

// Pops the next upcall off the kernel data's upcall queue and invokes it, or
// does nothing if the upcall queue was entry. The return value indicates
// whether an upcall was run. Panics if no kernel data is present.
fn invoke_next_upcall() -> bool {
    let option_queue_entry =
        with_kernel_data(|option_kernel_data| option_kernel_data.unwrap().upcall_queue.pop_front());
    match option_queue_entry {
        None => false,
        Some(queue_entry) => {
            unsafe {
                queue_entry.upcall.invoke(queue_entry.args);
            }
            true
        }
    }
}
