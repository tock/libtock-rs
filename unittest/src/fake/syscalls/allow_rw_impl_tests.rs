use crate::{fake, ExpectedSyscall, SyscallLogEntry};
use fake::syscalls::allow_rw_impl::*;
use libtock_platform::{return_variant, ErrorCode};
use std::convert::TryInto;
use std::panic::catch_unwind;

// TODO: Add a TestDriver, and add tests that use a driver:
// 1. A test that passes buffers to the driver and retrieves them.
// 2. A test with a driver that doesn't swap buffers (i.e. one that maintains a
//    longer list of buffers).
// 3. Fuzz tests
// 4. Test the driver error handling code.

// Tests calls that do not match the expected system call.
#[test]
fn expected_wrong() {
    let kernel = fake::Kernel::new();

    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: 1,
        command_id: 2,
        argument0: 3,
        argument1: 4,
        override_return: None,
    });
    assert!(catch_unwind(|| unsafe {
        allow_rw(1u32.into(), 2u32.into(), 0u32.into(), 0u32.into())
    })
    .expect_err("failed to catch wrong syscall class")
    .downcast_ref::<String>()
    .expect("wrong panic payload type")
    .contains("but Read-Write Allow was called instead"));

    kernel.add_expected_syscall(ExpectedSyscall::AllowRw {
        driver_num: 1,
        buffer_num: 2,
        return_error: None,
    });
    assert!(catch_unwind(|| unsafe {
        allow_rw(7u32.into(), 2u32.into(), 0u32.into(), 0u32.into())
    })
    .expect_err("failed to catch wrong driver number")
    .downcast_ref::<String>()
    .expect("wrong panic payload type")
    .contains("expected different driver_num"));

    kernel.add_expected_syscall(ExpectedSyscall::AllowRw {
        driver_num: 1,
        buffer_num: 2,
        return_error: None,
    });
    assert!(catch_unwind(|| unsafe {
        allow_rw(1u32.into(), 7u32.into(), 0u32.into(), 0u32.into())
    })
    .expect_err("failed to catch wrong buffer number")
    .downcast_ref::<String>()
    .expect("wrong panic payload type")
    .contains("expected different buffer_num"));
}

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    let [r0, r1, r2, r3] = unsafe { allow_rw(7u32.into(), 1u32.into(), 0u32.into(), 0u32.into()) };
    assert_eq!(
        r0.try_into(),
        Ok(Into::<u32>::into(return_variant::FAILURE_2_U32))
    );
    assert_eq!(r1.try_into(), Ok(ErrorCode::NoDevice as u32));
    assert_eq!(r2.try_into(), Ok(0u32));
    assert_eq!(r3.try_into(), Ok(0u32));
}

#[test]
fn no_kernel() {
    let result =
        catch_unwind(|| unsafe { allow_rw(1u32.into(), 1u32.into(), 0u32.into(), 0u32.into()) });
    assert!(result
        .expect_err("failed to catch missing kernel")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("no fake::Kernel exists"));
}

#[test]
fn syscall_log() {
    let kernel = fake::Kernel::new();
    // We want to pass a buffer of nonzero length to verify the length is logged
    // correctly.
    let buffer = [0; 3];
    unsafe {
        allow_rw(
            1u32.into(),
            2u32.into(),
            buffer.as_ptr().into(),
            buffer.len().into(),
        );
    }
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::AllowRw {
            driver_num: 1,
            buffer_num: 2,
            len: 3,
        }]
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn too_large_buffer_number() {
    let _kernel = fake::Kernel::new();
    let result = catch_unwind(|| unsafe {
        allow_rw(
            1u32.into(),
            (u32::MAX as usize + 1).into(),
            0u32.into(),
            0u32.into(),
        )
    });
    assert!(result
        .expect_err("failed to catch too-large buffer number")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("Too large buffer number"));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn too_large_driver_number() {
    let _kernel = fake::Kernel::new();
    let result = catch_unwind(|| unsafe {
        allow_rw(
            (u32::MAX as usize + 1).into(),
            1u32.into(),
            0u32.into(),
            0u32.into(),
        )
    });
    assert!(result
        .expect_err("failed to catch too-large driver number")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("Too large driver number"));
}
