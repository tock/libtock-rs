use crate::kernel::raw_syscalls_impl::assert_valid;
use crate::kernel::yield_impl::*;
use crate::{fake, ExpectedSyscall, SyscallLogEntry};
use libtock_platform::{RawSyscalls, YieldNoWaitReturn};
use std::panic::catch_unwind;

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
    assert_valid(return_value);
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
    assert_valid(return_value);
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

    // Test yield_wait with a skipped upcall in an expected syscall.
    kernel.add_expected_syscall(ExpectedSyscall::YieldWait { skip_upcall: true });
    yield_wait();
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
            catch_unwind(|| unsafe { fake::Kernel::yield1([(u32::MAX as usize + 1).into()]) });
        assert!(result
            .expect_err("failed to catch too large yield ID")
            .downcast_ref::<String>()
            .expect("wrong panic payload type")
            .contains("too-large Yield ID"));
    }

    // Call yield-no-wait through yield1, which is not valid.
    let result = catch_unwind(|| unsafe { fake::Kernel::yield1([0u32.into()]) });
    assert!(result
        .expect_err("failed to catch yield-no-wait without arg")
        .downcast_ref::<&'static str>()
        .expect("wrong panic payload type")
        .contains("yield-no-wait called without an argument"));

    // Test a successful invocation of yield-wait.
    kernel.add_expected_syscall(ExpectedSyscall::YieldWait { skip_upcall: true });
    unsafe {
        fake::Kernel::yield1([1u32.into()]);
    }
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldWait]);

    // Call yield1 with a yield ID that is unknown but which fits in a u32.
    let result = catch_unwind(|| unsafe { fake::Kernel::yield1([2u32.into()]) });
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
            fake::Kernel::yield2([(u32::MAX as usize + 1).into(), 0u32.into()])
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
        fake::Kernel::yield2([0u32.into(), return_value.as_mut_ptr().into()]);
    }
    let return_value = unsafe { return_value.assume_init() };
    assert_valid(return_value);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);
    assert_eq!(return_value, YieldNoWaitReturn::Upcall);

    // Call yield-wait through yield2, which should be rejected.
    let result = catch_unwind(|| unsafe { fake::Kernel::yield2([1u32.into(), 0u32.into()]) });
    assert!(result
        .expect_err("failed to catch yield-wait with arg")
        .downcast_ref::<&'static str>()
        .expect("wrong panic payload type")
        .contains("yield-wait called with an argument"));

    // Call yield2 with a yield ID that is unknown but which fits in a u32.
    let result = catch_unwind(|| unsafe { fake::Kernel::yield2([2u32.into(), 0u32.into()]) });
    assert!(result
        .expect_err("failed to catch incorrect yield ID -- new ID added?")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("unknown yield ID"));
}
