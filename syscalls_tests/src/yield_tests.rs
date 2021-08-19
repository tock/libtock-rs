//! Tests for implementations of Yield system calls in
//! `libtock_platform::Syscalls`.

use libtock_platform::{Syscalls, YieldNoWaitReturn};
use libtock_unittest::{fake, ExpectedSyscall, SyscallLogEntry};

// Tests yield_no_wait with an upcall executed.
#[test]
fn no_wait_upcall() {
    use YieldNoWaitReturn::Upcall;
    let kernel = fake::Kernel::new();
    kernel.add_expected_syscall(ExpectedSyscall::YieldNoWait {
        override_return: Some(Upcall),
    });
    assert_eq!(fake::Syscalls::yield_no_wait(), Upcall);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);
}

// Tests yield_no_wait with no upcall executed.
#[test]
fn no_wait_no_upcall() {
    use YieldNoWaitReturn::NoUpcall;
    let kernel = fake::Kernel::new();
    kernel.add_expected_syscall(ExpectedSyscall::YieldNoWait {
        override_return: Some(NoUpcall),
    });
    assert_eq!(fake::Syscalls::yield_no_wait(), NoUpcall);
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldNoWait]);
}

// Tests yield_wait.
#[test]
fn wait() {
    let kernel = fake::Kernel::new();
    kernel.add_expected_syscall(ExpectedSyscall::YieldWait { skip_upcall: true });
    fake::Syscalls::yield_wait();
    assert_eq!(kernel.take_syscall_log(), [SyscallLogEntry::YieldWait]);
}
