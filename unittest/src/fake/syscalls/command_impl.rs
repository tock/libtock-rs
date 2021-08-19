//! `fake::Kernel`'s implementation of the Command system call.

use crate::kernel_data::with_kernel_data;
use crate::{command_return, ExpectedSyscall, SyscallLogEntry};
use libtock_platform::{ErrorCode, Register};
use std::convert::TryInto;

pub(super) fn command(
    driver_id: Register,
    command_id: Register,
    argument0: Register,
    argument1: Register,
) -> [Register; 4] {
    let driver_id = driver_id.try_into().expect("Too large driver ID");
    let command_id = command_id.try_into().expect("Too large command ID");
    let argument0 = argument0.try_into().expect("Too large argument 0");
    let argument1 = argument1.try_into().expect("Too large argument 1");

    let (driver, override_return) = with_kernel_data(|option_kernel_data| {
        let kernel_data = option_kernel_data.expect("Command called but no fake::Kernel exists");

        kernel_data.syscall_log.push(SyscallLogEntry::Command {
            driver_id,
            command_id,
            argument0,
            argument1,
        });

        // Check for an expected syscall entry. Sets override_return to None if
        // the expected syscall queue is empty or if it expected this syscall
        // but did not specify a return override. Panics if a different syscall
        // was expected (either a non-Command syscall, or a Command call with
        // different arguments).
        let override_return = match kernel_data.expected_syscalls.pop_front() {
            None => None,
            Some(ExpectedSyscall::Command {
                driver_id: expected_driver_id,
                command_id: expected_command_id,
                argument0: expected_argument0,
                argument1: expected_argument1,
                override_return,
            }) => {
                assert_eq!(
                    driver_id, expected_driver_id,
                    "expected different driver_id"
                );
                assert_eq!(
                    command_id, expected_command_id,
                    "expected different command_id"
                );
                assert_eq!(
                    argument0, expected_argument0,
                    "expected different argument0"
                );
                assert_eq!(
                    argument1, expected_argument1,
                    "expected different argument1"
                );
                override_return
            }
            Some(expected_syscall) => expected_syscall.panic_wrong_call("Command"),
        };

        let driver = kernel_data
            .drivers
            .get(&driver_id)
            .map(|driver_data| driver_data.driver.clone());

        (driver, override_return)
    });

    // Call the driver if one is present. If not, return NoDevice as required by
    // TRD 104.
    let driver_return = match driver {
        Some(driver) => driver.command(command_id, argument0, argument1),
        None => command_return::failure(ErrorCode::NoDevice),
    };

    // Convert the override return value (or the driver return value if no
    // override is present) into the representative register values.
    let (return_variant, r1, r2, r3) = override_return.unwrap_or(driver_return).raw_values();
    let r0: u32 = return_variant.into();
    [r0.into(), r1.into(), r2.into(), r3.into()]
}
