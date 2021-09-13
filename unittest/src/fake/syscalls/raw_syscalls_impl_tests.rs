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
            driver_number: 1,
            buffer_number: 2,
            len: 0,
        }]
    );
}

// TODO: Implement Read-Write Allow.

// TODO: Move the syscall4 Command test here.

// TODO: Implement Exit.

// TODO: Implement Memop.

// TODO: Implement Subscribe.

// TODO: Move the yield1 and yield2 tests here.
