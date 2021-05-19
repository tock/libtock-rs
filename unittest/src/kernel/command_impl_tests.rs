use super::command_impl::*;
use crate::{command_return, fake, ExpectedSyscall, SyscallLogEntry};
use libtock_platform::{return_variant, RawSyscalls, ReturnVariant};
use std::convert::TryInto;
use std::panic::catch_unwind;

// TODO: When another system call is implemented, add a test for the case
// where a different system call class is expected.

// Tests command with expected syscalls that don't match this command call.
#[test]
fn expected_wrong_command() {
    let kernel = fake::Kernel::new("expected_wrong_command");
    let expected_syscall = ExpectedSyscall::Command {
        driver_id: 1,
        command_id: 1,
        argument0: 1,
        argument1: 1,
        override_return: None,
    };

    kernel.add_expected_syscall(expected_syscall);
    assert!(
        catch_unwind(|| command(2u32.into(), 1u32.into(), 1u32.into(), 1u32.into()))
            .expect_err("failed to catch wrong driver_id")
            .downcast_ref::<String>()
            .expect("wrong panic payload type")
            .contains("expected different driver_id")
    );

    kernel.add_expected_syscall(expected_syscall);
    assert!(
        catch_unwind(|| command(1u32.into(), 2u32.into(), 1u32.into(), 1u32.into()))
            .expect_err("failed to catch wrong command_id")
            .downcast_ref::<String>()
            .expect("wrong panic payload type")
            .contains("expected different command_id")
    );

    kernel.add_expected_syscall(expected_syscall);
    assert!(
        catch_unwind(|| command(1u32.into(), 1u32.into(), 2u32.into(), 1u32.into()))
            .expect_err("failed to catch wrong argument0")
            .downcast_ref::<String>()
            .expect("wrong panic payload type")
            .contains("expected different argument0")
    );

    kernel.add_expected_syscall(expected_syscall);
    assert!(
        catch_unwind(|| command(1u32.into(), 1u32.into(), 1u32.into(), 2u32.into()))
            .expect_err("failed to catch wrong argument1")
            .downcast_ref::<String>()
            .expect("wrong panic payload type")
            .contains("expected different argument1")
    );
}

#[test]
fn no_kernel() {
    let result = catch_unwind(|| command(1u32.into(), 1u32.into(), 0u32.into(), 0u32.into()));
    assert!(result
        .expect_err("failed to catch missing kernel")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("no fake::Kernel exists"));
}

#[test]
fn override_return() {
    let kernel = fake::Kernel::new("override_return");
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: 1,
        command_id: 2,
        argument0: 3,
        argument1: 4,
        override_return: Some(command_return::success_3_u32(1, 2, 3)),
    });
    let [r0, r1, r2, r3] = command(1u32.into(), 2u32.into(), 3u32.into(), 4u32.into());
    let r0: u32 = r0.try_into().expect("too large r0");
    let r1: u32 = r1.try_into().expect("too large r1");
    let r2: u32 = r2.try_into().expect("too large r2");
    let r3: u32 = r3.try_into().expect("too large r3");
    let return_variant: ReturnVariant = r0.into();
    assert_eq!(return_variant, return_variant::SUCCESS_3_U32);
    assert_eq!(r1, 1);
    assert_eq!(r2, 2);
    assert_eq!(r3, 3);
}

// Test that fake::Kernel's implementation of RawSyscalls correctly forwards
// a command to `command`.
// TODO: Migrate into raw_syscalls_impl.rs when the other system calls are
// completed, to avoid git conflicts.
#[test]
fn syscall4() {
    let kernel = fake::Kernel::new("syscall4");
    unsafe {
        fake::Kernel::syscall4::<2>([1u32.into(), 2u32.into(), 3u32.into(), 4u32.into()]);
    }
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::Command {
            driver_id: 1,
            command_id: 2,
            argument0: 3,
            argument1: 4,
        }]
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn too_large_argument0() {
    let _kernel = fake::Kernel::new("too_large_argument0");
    let result = catch_unwind(|| {
        command(
            1u32.into(),
            1u32.into(),
            (u32::MAX as usize + 1).into(),
            0u32.into(),
        )
    });
    assert!(result
        .expect_err("failed to catch too-large argument0")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("Too large argument 0"));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn too_large_argument1() {
    let _kernel = fake::Kernel::new("too_large_argument1");
    let result = catch_unwind(|| {
        command(
            1u32.into(),
            1u32.into(),
            0u32.into(),
            (u32::MAX as usize + 1).into(),
        )
    });
    assert!(result
        .expect_err("failed to catch too-large argument1")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("Too large argument 1"));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn too_large_command_id() {
    let _kernel = fake::Kernel::new("too_large_command_id");
    let result = catch_unwind(|| {
        command(
            1u32.into(),
            (u32::MAX as usize + 1).into(),
            0u32.into(),
            0u32.into(),
        )
    });
    assert!(result
        .expect_err("failed to catch too-large command ID")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("Too large command ID"));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn too_large_driver_id() {
    let _kernel = fake::Kernel::new("too_large_driver_id");
    let result = catch_unwind(|| {
        command(
            (u32::MAX as usize + 1).into(),
            1u32.into(),
            0u32.into(),
            0u32.into(),
        )
    });
    assert!(result
        .expect_err("failed to catch too-large driver ID")
        .downcast_ref::<String>()
        .expect("wrong panic payload type")
        .contains("Too large driver ID"));
}

// TODO: When driver support is added, add a test that tests driver support.
