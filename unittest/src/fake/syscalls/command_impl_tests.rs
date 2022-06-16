use super::command_impl::*;
use crate::{command_return, fake, DriverInfo, ExpectedSyscall, SyscallLogEntry};
use libtock_platform::{
    return_variant, syscall_class, CommandReturn, ErrorCode, RawSyscalls, ReturnVariant,
};
use std::convert::TryInto;
use std::panic::catch_unwind;

// TODO: When another system call is implemented, add a test for the case
// where a different system call class is expected.

#[test]
fn driver_support() {
    let kernel = fake::Kernel::new();

    // Call command for a nonexistent driver.
    let [r0, r1, _, _] = command(42u32.into(), 1u32.into(), 0u32.into(), 0u32.into());
    assert_eq!(
        r0.try_into(),
        Ok(Into::<u32>::into(return_variant::FAILURE))
    );
    assert_eq!(r1.try_into(), Ok(ErrorCode::NoDevice as u32));

    // A mock driver that returns a fixed value.
    // TODO: This is growing every time we add a new required method to Driver.
    // Once we have fake driver inside `crate::fake` (e.g. a fake LowLevelDebug
    // driver), we should remove MockDriver and replace it with the fake driver,
    // so we have 1 fewer Driver implementations to maintain.
    struct MockDriver;
    impl fake::SyscallDriver for MockDriver {
        fn info(&self) -> DriverInfo {
            DriverInfo::new(42)
        }
        fn command(&self, _command_id: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
            command_return::success_3_u32(1, 2, 3)
        }
    }

    // Call command with the mock driver.
    let driver = std::rc::Rc::new(MockDriver);
    kernel.add_driver(&driver);
    let [r0, r1, r2, r3] = command(42u32.into(), 0u32.into(), 0u32.into(), 0u32.into());
    assert_eq!(
        r0.try_into(),
        Ok(Into::<u32>::into(return_variant::SUCCESS_3_U32))
    );
    assert_eq!(r1.try_into(), Ok(1u32));
    assert_eq!(r2.try_into(), Ok(2u32));
    assert_eq!(r3.try_into(), Ok(3u32));
}

// Tests command with expected syscalls that don't match this command call.
#[test]
fn expected_wrong_command() {
    let kernel = fake::Kernel::new();
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
    let kernel = fake::Kernel::new();
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
    let kernel = fake::Kernel::new();
    unsafe {
        fake::Syscalls::syscall4::<{ syscall_class::COMMAND }>([
            1u32.into(),
            2u32.into(),
            3u32.into(),
            4u32.into(),
        ]);
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
    let _kernel = fake::Kernel::new();
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
    let _kernel = fake::Kernel::new();
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
    let _kernel = fake::Kernel::new();
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
    let _kernel = fake::Kernel::new();
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
