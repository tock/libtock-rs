use crate::kernel_data::with_kernel_data;
use crate::{fake, ExpectedSyscall, SyscallLogEntry};

#[test]
fn expected_syscall_queue() {
    use libtock_platform::YieldNoWaitReturn::Upcall;
    use std::matches;
    use ExpectedSyscall::YieldNoWait;
    let kernel = fake::Kernel::new();
    with_kernel_data(|kernel_data| assert!(kernel_data.unwrap().expected_syscalls.is_empty()));
    kernel.add_expected_syscall(YieldNoWait {
        override_return: None,
    });
    kernel.add_expected_syscall(YieldNoWait {
        override_return: Some(Upcall),
    });
    with_kernel_data(|kernel_data| {
        let expected_syscalls = &mut kernel_data.unwrap().expected_syscalls;
        assert!(matches!(
            expected_syscalls.pop_front(),
            Some(YieldNoWait {
                override_return: None
            })
        ));
        assert!(matches!(
            expected_syscalls.pop_front(),
            Some(YieldNoWait {
                override_return: Some(Upcall)
            })
        ));
        assert!(expected_syscalls.is_empty());
    });
}

#[test]
fn syscall_log() {
    use SyscallLogEntry::{YieldNoWait, YieldWait};
    let kernel = fake::Kernel::new();
    assert_eq!(kernel.take_syscall_log(), []);
    with_kernel_data(|kernel_data| {
        let syscall_log = &mut kernel_data.unwrap().syscall_log;
        syscall_log.push(YieldNoWait);
        syscall_log.push(YieldWait);
    });
    assert_eq!(kernel.take_syscall_log(), [YieldNoWait, YieldWait]);
    assert_eq!(kernel.take_syscall_log(), []);
}
