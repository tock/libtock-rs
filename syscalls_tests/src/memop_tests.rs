//! Tests for the Memop system call implementation in
//! `libtock_platform::Syscalls`.

use libtock_platform::{ErrorCode, Syscalls};
use libtock_unittest::{fake, ExpectedSyscall, SyscallLogEntry};

#[test]
fn memop() {
    let kernel = fake::Kernel::new();
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 1,
        argument0: 3.into(),
        return_error: Some(ErrorCode::NoMem),
    });
    assert_eq!(
        unsafe { fake::Syscalls::memop_sbrk(3) },
        Err(ErrorCode::NoMem)
    );
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::Memop {
            memop_num: 1,
            argument0: 3.into(),
        }]
    );
}

#[test]
fn brk_test() {
    let kernel = fake::Kernel::new();
    let fake_mem_buf = [0; 8];
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 0,
        argument0: fake_mem_buf.as_ptr().into(),
        return_error: None,
    });
    assert_eq!(
        unsafe { fake::Syscalls::memop_brk(fake_mem_buf.as_ptr()) },
        Ok(())
    );
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::Memop {
            memop_num: 0,
            argument0: fake_mem_buf.as_ptr().into(),
        }]
    );
}

#[test]
fn sbrk_test() {
    let kernel = fake::Kernel::new();
    let fake_mem_buf = [0; 8];
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 0,
        argument0: fake_mem_buf.as_ptr().into(),
        return_error: None,
    });
    assert_eq!(
        unsafe { fake::Syscalls::memop_brk(fake_mem_buf.as_ptr()) },
        Ok(())
    );
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 1,
        argument0: 4.into(),
        return_error: None,
    });
    assert_eq!(
        unsafe { fake::Syscalls::memop_sbrk(4) },
        Ok((&fake_mem_buf[4]) as *const u8)
    );
}

#[test]
fn increment_brk_test() {
    let kernel = fake::Kernel::new();
    let fake_mem_buf = [0; 8];
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 0,
        argument0: fake_mem_buf.as_ptr().into(),
        return_error: None,
    });
    assert_eq!(
        unsafe { fake::Syscalls::memop_brk(fake_mem_buf.as_ptr()) },
        Ok(())
    );
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 1,
        argument0: 4.into(),
        return_error: None,
    });
    assert_eq!(
        fake::Syscalls::memop_increment_brk(4),
        Ok((&fake_mem_buf[4]) as *const u8)
    );
}

#[test]
fn app_ram_start_test() {
    let kernel = fake::Kernel::new();
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 2,
        argument0: 0.into(),
        return_error: None,
    });
    assert!(fake::Syscalls::memop_app_ram_start().is_ok());
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::Memop {
            memop_num: 2,
            argument0: 0.into(),
        }]
    );
}

#[test]
fn debug_stack_start_test() {
    let kernel = fake::Kernel::new();
    let fake_stack = [0; 8];
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 10,
        argument0: fake_stack.as_ptr().into(),
        return_error: None,
    });
    assert!(fake::Syscalls::memop_debug_stack_start(fake_stack.as_ptr()).is_ok());
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::Memop {
            memop_num: 10,
            argument0: fake_stack.as_ptr().into(),
        }]
    );
}

#[test]
fn debug_heap_start_test() {
    let kernel = fake::Kernel::new();
    let fake_heap = [0; 8];
    kernel.add_expected_syscall(ExpectedSyscall::Memop {
        memop_num: 11,
        argument0: fake_heap.as_ptr().into(),
        return_error: None,
    });
    assert!(fake::Syscalls::memop_debug_heap_start(fake_heap.as_ptr()).is_ok());
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::Memop {
            memop_num: 11,
            argument0: fake_heap.as_ptr().into(),
        }]
    );
}
