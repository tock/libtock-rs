//! Tests for the Command system call implementation in
//! `libtock_platform::Syscalls`.

use libtock_platform::Syscalls;
use libtock_unittest::{command_return, fake, ExpectedSyscall, SyscallLogEntry};

#[test]
fn command() {
    let kernel = fake::Kernel::new();
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: 1,
        command_id: 2,
        argument0: 3,
        argument1: 4,
        override_return: Some(command_return::success_3_u32(1, 2, 3)),
    });
    assert_eq!(
        fake::Syscalls::command(1, 2, 3, 4).get_success_3_u32(),
        Some((1, 2, 3))
    );
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
