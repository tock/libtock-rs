use crate::kernel_data::with_kernel_data;
use crate::upcall::{Upcall, UpcallId, UpcallQueueEntry};
use crate::{fake, ExpectedSyscall, SyscallLogEntry};
use libtock_platform::{RawSyscalls, YieldNoWaitReturn};
use std::panic::catch_unwind;

use fake::syscalls::yield_impl::*;

// Upcall function that copies its arguments into the [u32; 3] pointed to by
// `output`. Used by multiple tests in this file.
unsafe extern "C" fn copy_args(
    arg0: u32,
    arg1: u32,
    arg2: u32,
    output: libtock_platform::Register,
) {
    let output: *mut [u32; 3] = output.into();
    unsafe {
        *output = [arg0, arg1, arg2];
    }
}

#[test]
fn yield_no_wait_test() {
    // Test calling yield_no_wait with no fake::Kernel present.
    let result = catch_unwind(|| {
        let mut return_value = core::mem::MaybeUninit::<YieldNoWaitReturn>::uninit();
        unsafe {
            yield_no_wait(return_value.as_mut_ptr());
        }
    });
    assert!(result
        .expect_err("failed to catch missing fake::Kernel")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("no fake::Kernel"));

    let kernel = fake::Kernel::new();

    // Test yield_no_wait with an empty upcall queue and empty expected syscall
    // queue.
    let mut return_value = core::mem::MaybeUninit::<YieldNoWaitReturn>::uninit();
    unsafe {
        yield_no_wait(return_value.as_mut_ptr());
    }
    let return_value = unsafe { return_value.assume_init() };
    fake::syscalls::assert_valid(return_value);
    assert_eq!(return_value, YieldNoWaitReturn::NoUpcall);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);

    // Test yield_no_wait with a return override in an expected syscall.
    kernel.add_expected_syscall(ExpectedSyscall::YieldNoWait {
        override_return: Some(YieldNoWaitReturn::Upcall),
    });
    let mut return_value = core::mem::MaybeUninit::<YieldNoWaitReturn>::uninit();
    unsafe {
        yield_no_wait(return_value.as_mut_ptr());
    }
    let return_value = unsafe { return_value.assume_init() };
    fake::syscalls::assert_valid(return_value);
    assert_eq!(return_value, YieldNoWaitReturn::Upcall);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);

    // Test yield_no_wait with a mismatched expected syscall.
    kernel.add_expected_syscall(ExpectedSyscall::YieldWait { skip_upcall: false });
    let result = catch_unwind(|| {
        let mut return_value = core::mem::MaybeUninit::<YieldNoWaitReturn>::uninit();
        unsafe {
            yield_no_wait(return_value.as_mut_ptr());
        }
    });
    assert!(result
        .expect_err("failed to catch mismatched expected syscall")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("yield-no-wait was called instead"));
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);

    // Upcall structures for using copy_args.
    let mut output_array = [0u32; 3];
    let upcall_id = UpcallId {
        driver_num: 1,
        subscribe_num: 2,
    };
    let upcall = Upcall {
        fn_pointer: Some(copy_args),
        data: (&mut output_array as *mut u32).into(),
    };

    // Test yield_no_wait with an upcall queued.
    with_kernel_data(|option_kernel_data| {
        option_kernel_data
            .unwrap()
            .upcall_queue
            .push_back(UpcallQueueEntry {
                args: (1, 2, 3),
                id: upcall_id,
                upcall,
            });
    });
    let mut return_value = core::mem::MaybeUninit::<YieldNoWaitReturn>::uninit();
    unsafe {
        yield_no_wait(return_value.as_mut_ptr());
    }
    assert_eq!(output_array, [1, 2, 3]);
    let return_value = unsafe { return_value.assume_init() };
    fake::syscalls::assert_valid(return_value);
    assert_eq!(return_value, YieldNoWaitReturn::Upcall);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);

    // Test yield_no_wait with an upcall queued and a return override in an
    // expected syscall.
    kernel.add_expected_syscall(ExpectedSyscall::YieldNoWait {
        override_return: Some(YieldNoWaitReturn::NoUpcall),
    });
    with_kernel_data(|option_kernel_data| {
        option_kernel_data
            .unwrap()
            .upcall_queue
            .push_back(UpcallQueueEntry {
                args: (4, 5, 6),
                id: upcall_id,
                upcall,
            });
    });
    let mut return_value = core::mem::MaybeUninit::<YieldNoWaitReturn>::uninit();
    unsafe {
        yield_no_wait(return_value.as_mut_ptr());
    }
    assert_eq!(output_array, [4, 5, 6]);
    let return_value = unsafe { return_value.assume_init() };
    fake::syscalls::assert_valid(return_value);
    assert_eq!(return_value, YieldNoWaitReturn::NoUpcall);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);
}

#[test]
fn yield_wait_test() {
    // Test calling yield_wait with no fake::Kernel present.
    assert!(catch_unwind(yield_wait)
        .expect_err("failed to catch missing fake::Kernel")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("no fake::Kernel"));

    let kernel = fake::Kernel::new();

    // Test yield_wait with a mismatched expected syscall.
    kernel.add_expected_syscall(ExpectedSyscall::YieldNoWait {
        override_return: None,
    });
    assert!(catch_unwind(yield_wait)
        .expect_err("failed to catch mismatched expected syscall")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("yield-wait was called instead"));
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldWait]);

    // Upcall structures for using copy_args.
    let mut output_array = [0u32; 3];
    let upcall_id = UpcallId {
        driver_num: 1,
        subscribe_num: 2,
    };
    let upcall = Upcall {
        fn_pointer: Some(copy_args),
        data: (&mut output_array as *mut u32).into(),
    };

    // Test yield_wait with a skipped upcall in an expected syscall.
    with_kernel_data(|option_kernel_data| {
        option_kernel_data
            .unwrap()
            .upcall_queue
            .push_back(UpcallQueueEntry {
                args: (1, 2, 3),
                id: upcall_id,
                upcall,
            });
    });
    kernel.add_expected_syscall(ExpectedSyscall::YieldWait { skip_upcall: true });
    yield_wait();
    assert_eq!(output_array, [0; 3]);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldWait]);

    // Test that yield_wait correctly invokes a queued upcall. The upcall was
    // queued for the previous test (which confirmed that skip_upcall works).
    yield_wait();
    assert_eq!(output_array, [1, 2, 3]);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldWait]);
}

// TODO: Move the yield1 and yield2 tests into a raw_syscalls_impl test module,
// once all system calls have been implemented.

#[test]
fn yield1() {
    let kernel = fake::Kernel::new();

    #[cfg(target_pointer_width = "64")]
    {
        let result =
            catch_unwind(|| unsafe { fake::Syscalls::yield1([(u32::MAX as usize + 1).into()]) });
        assert!(result
            .expect_err("failed to catch too large yield ID")
            .downcast_ref::<String>()
            .expect("wrong panic payload type")
            .contains("too-large Yield ID"));
    }

    // Call yield-no-wait through yield1, which is not valid.
    let result = catch_unwind(|| unsafe { fake::Syscalls::yield1([0u32.into()]) });
    assert!(result
        .expect_err("failed to catch yield-no-wait without arg")
        .downcast_ref::<&'static str>()
        .expect("wrong panic payload type")
        .contains("yield-no-wait called without an argument"));

    // Test a successful invocation of yield-wait.
    kernel.add_expected_syscall(ExpectedSyscall::YieldWait { skip_upcall: true });
    unsafe {
        fake::Syscalls::yield1([1u32.into()]);
    }
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldWait]);

    // Call yield1 with a yield ID that is unknown but which fits in a u32.
    let result = catch_unwind(|| unsafe { fake::Syscalls::yield1([2u32.into()]) });
    assert!(result
        .expect_err("failed to catch incorrect yield ID -- new ID added?")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("unknown yield ID"));
}

// Tests RawSyscalls::yield2's handling of bad yield IDs.
#[test]
fn yield2() {
    let kernel = fake::Kernel::new();

    #[cfg(target_pointer_width = "64")]
    {
        let result = catch_unwind(|| unsafe {
            fake::Syscalls::yield2([(u32::MAX as usize + 1).into(), 0u32.into()])
        });
        assert!(result
            .expect_err("failed to catch too large yield ID")
            .downcast_ref::<String>()
            .expect("wrong panic payload type")
            .contains("too-large Yield ID"));
    }

    // Test a successful invocation of yield-no-wait.
    kernel.add_expected_syscall(ExpectedSyscall::YieldNoWait {
        override_return: Some(YieldNoWaitReturn::Upcall),
    });
    let mut return_value = core::mem::MaybeUninit::<YieldNoWaitReturn>::uninit();
    unsafe {
        fake::Syscalls::yield2([0u32.into(), return_value.as_mut_ptr().into()]);
    }
    let return_value = unsafe { return_value.assume_init() };
    fake::syscalls::assert_valid(return_value);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);
    assert_eq!(return_value, YieldNoWaitReturn::Upcall);

    // Call yield-wait through yield2, which should be rejected.
    let result = catch_unwind(|| unsafe { fake::Syscalls::yield2([1u32.into(), 0u32.into()]) });
    assert!(result
        .expect_err("failed to catch yield-wait with arg")
        .downcast_ref::<&'static str>()
        .expect("wrong panic payload type")
        .contains("yield-wait called with an argument"));

    // Call yield2 with a yield ID that is unknown but which fits in a u32.
    let result = catch_unwind(|| unsafe { fake::Syscalls::yield2([2u32.into(), 0u32.into()]) });
    assert!(result
        .expect_err("failed to catch incorrect yield ID -- new ID added?")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("unknown yield ID"));
}
