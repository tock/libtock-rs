// These tests verify the RawSyscalls implementation routes system calls to the
// fake implementations (e.g. command(), yield_wait, etc) correctly. It does not
// test the fake syscall implementations themselves, as they have their own unit
// tests.

use crate::{fake, SyscallLogEntry};
use libtock_platform::{syscall_class, RawSyscalls};

#[test]
fn allow_ro() {
    let kernel = fake::Kernel::new();
    unsafe {
        fake::Syscalls::syscall4::<{ syscall_class::ALLOW_RO }>([
            1u32.into(),
            2u32.into(),
            0u32.into(),
            0u32.into(),
        ]);
    }
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::AllowRo {
            driver_num: 1,
            buffer_num: 2,
            len: 0,
        }]
    );
}

#[test]
fn allow_rw() {
    let kernel = fake::Kernel::new();
    unsafe {
        fake::Syscalls::syscall4::<{ syscall_class::ALLOW_RW }>([
            1u32.into(),
            2u32.into(),
            0u32.into(),
            0u32.into(),
        ]);
    }
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::AllowRw {
            driver_num: 1,
            buffer_num: 2,
            len: 0,
        }]
    );
}

// TODO: Move the syscall4 Command test here.

// TODO: Implement Exit.

#[test]
fn memop() {
    let kernel = fake::Kernel::new();
    unsafe {
        fake::Syscalls::syscall2::<{ syscall_class::MEMOP }>([1u32.into(), 2u32.into()]);
        fake::Syscalls::syscall1::<{ syscall_class::MEMOP }>([2u32.into()]);
    }
    assert_eq!(
        kernel.take_syscall_log(),
        [
            SyscallLogEntry::Memop {
                memop_num: 1,
                argument0: 2.into(),
            },
            SyscallLogEntry::Memop {
                memop_num: 2,
                argument0: 0.into(),
            }
        ]
    );
}

// TODO: Implement Subscribe.

// TODO: Move the yield1 and yield2 tests here.
