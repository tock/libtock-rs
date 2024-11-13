use super::subscribe_impl::*;
use crate::{fake, ExpectedSyscall, SyscallLogEntry};
use libtock_platform::{return_variant, syscall_class, ErrorCode, RawSyscalls, Register};
use std::convert::TryInto;
use std::panic::catch_unwind;

// TODO: Once a fake driver that supports upcalls is added, add the following
// test cases:
// 1. A test with a subscribe_id that is too large.
// 2. A test that should pass -- and verify the upcall is set correctly.
// 3. A test that verifies that upcalls are correctly cleared from the queue
//    when they are replaced by a subsequent Subscribe call.

// Tests Subscribe calls that do not match the expected syscall.
#[test]
fn expected_wrong() {
    let kernel = fake::Kernel::new();

    // Test with a non-Subscribe expected syscall.
    kernel.add_expected_syscall(ExpectedSyscall::YieldWait { skip_upcall: false });
    let result = catch_unwind(|| unsafe {
        subscribe(1u32.into(), 2u32.into(), 0usize.into(), 0usize.into())
    });
    assert!(result
        .expect_err("failed to catch wrong syscall")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("but Subscribe was called instead"));

    let expected_syscall = ExpectedSyscall::Subscribe {
        driver_num: 1,
        subscribe_num: 2,
        skip_with_error: None,
    };

    // Tests with an incorrect driver number
    kernel.add_expected_syscall(expected_syscall);
    let result = catch_unwind(|| unsafe {
        subscribe(7u32.into(), 2u32.into(), 0usize.into(), 0usize.into())
    });
    assert!(result
        .expect_err("failed to catch wrong driver number")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("expected different driver number"));

    // Tests with an incorrect subscribe number
    kernel.add_expected_syscall(expected_syscall);
    let result = catch_unwind(|| unsafe {
        subscribe(1u32.into(), 7u32.into(), 0usize.into(), 0usize.into())
    });
    assert!(result
        .expect_err("failed to catch wrong subscribe number")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("expected different subscribe number"));
}

// Test Subscribe with a driver number that does not exist.
#[test]
fn missing_driver() {
    let _kernel = fake::Kernel::new();
    let [r0, r1, r2, r3] =
        unsafe { subscribe(1u32.into(), 2u32.into(), 0usize.into(), 0usize.into()) };
    let (r0, r1, r2, r3): (u32, u32, usize, usize) = (
        r0.try_into().expect("too large r0"),
        r1.try_into().expect("too large r1"),
        r2.into(),
        r3.into(),
    );
    assert_eq!(r0, return_variant::FAILURE_2_U32.into());
    assert_eq!(r1, ErrorCode::NoDevice as u32);
    assert_eq!(r2, 0);
    assert_eq!(r3, 0);
}

#[test]
fn no_kernel() {
    let result = catch_unwind(|| unsafe {
        subscribe(1u32.into(), 2u32.into(), 0usize.into(), 0usize.into())
    });
    assert!(result
        .expect_err("failed to catch missing kernel")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("no fake::Kernel exists"));
}

#[test]
fn skip_with_error() {
    let kernel = fake::Kernel::new();
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 1,
        subscribe_num: 2,
        skip_with_error: Some(ErrorCode::NoAck),
    });
    unsafe extern "C" fn upcall_fn(_: u32, _: u32, _: u32, _: Register) {}
    // Convert to a raw pointer to get a stable address.
    let upcall_fn_ptr = upcall_fn as *const ();
    let [r0, r1, r2, r3] = unsafe {
        subscribe(
            1u32.into(),
            2u32.into(),
            upcall_fn_ptr.into(),
            1234usize.into(),
        )
    };
    let (r0, r1, r2, r3): (u32, u32, *const (), usize) = (
        r0.try_into().expect("too large r0"),
        r1.try_into().expect("too large r1"),
        r2.into(),
        r3.into(),
    );
    assert_eq!(r0, return_variant::FAILURE_2_U32.into());
    assert_eq!(r1, ErrorCode::NoAck as u32);
    assert_eq!(r2, upcall_fn_ptr);
    assert_eq!(r3, 1234);
}

// TODO: Move the syscall4_subscribe test into raw_syscalls_impl_tests.rs, once
// raw_syscalls_impl_tests.rs has been created.

#[test]
fn syscall4_subscribe() {
    let kernel = fake::Kernel::new();
    unsafe {
        fake::Syscalls::syscall4::<{ syscall_class::SUBSCRIBE }>([
            1u32.into(),
            2u32.into(),
            0u32.into(),
            0u32.into(),
        ]);
    }
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::Subscribe {
            driver_num: 1,
            subscribe_num: 2,
        }]
    );
}

// Tests Subscribe with too large inputs (driver_num and subscribe_num)
#[cfg(target_pointer_width = "64")]
#[test]
fn too_large_inputs() {
    let _kernel = fake::Kernel::new();

    let result = catch_unwind(|| unsafe {
        subscribe(
            (u32::MAX as usize + 1).into(),
            2u32.into(),
            0usize.into(),
            0usize.into(),
        )
    });
    assert!(result
        .expect_err("failed to catch too-large driver number")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("Too large driver number"));

    let result = catch_unwind(|| unsafe {
        subscribe(
            1u32.into(),
            (u32::MAX as usize + 1).into(),
            0usize.into(),
            0usize.into(),
        )
    });
    assert!(result
        .expect_err("failed to catch too-large subscribe number")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("Too large subscribe number"));
}
