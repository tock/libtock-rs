use super::*;
use libtock_platform::ErrorCode;
use libtock_unittest::{command_return, fake, ExpectedSyscall};

#[test]
fn driver_check() {
    // Create a new fake kernel for the current thread, replacing avy previous one.
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);

    assert!(LowLevelDebug::<fake::Syscalls>::driver_check().is_success());
    assert_eq!(driver.take_messages(), []);
}

#[test]
fn print_alert_code() {
    // Create a new fake kernel for the current thread, replacing avy previous one.
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);

    assert!(LowLevelDebug::<fake::Syscalls>::print_alert_code(AlertCode::Panic).is_success());
    assert!(
        LowLevelDebug::<fake::Syscalls>::print_alert_code(AlertCode::WrongLocation).is_success()
    );
    assert_eq!(
        driver.take_messages(),
        [
            fake::Message::AlertCode(0x01),
            fake::Message::AlertCode(0x02)
        ]
    );
}

#[test]
fn print_1() {
    // Create a new fake kernel for the current thread, replacing avy previous one.
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);

    assert!(LowLevelDebug::<fake::Syscalls>::print_1(42).is_success());
    assert_eq!(driver.take_messages(), [fake::Message::Print1(42)]);
}

#[test]
fn print_2() {
    // Create a new fake kernel for the current thread, replacing avy previous one.
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);

    assert!(LowLevelDebug::<fake::Syscalls>::print_2(42, 27).is_success());
    assert!(LowLevelDebug::<fake::Syscalls>::print_2(29, 43).is_success());
    assert_eq!(
        driver.take_messages(),
        [fake::Message::Print2(42, 27), fake::Message::Print2(29, 43)]
    );
}

#[test]
fn failed_print() {
    // Create a new fake kernel for the current thread, replacing avy previous one.
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_ID,
        command_id: PRINT_1,
        argument0: 72,
        argument1: 0,
        override_return: Some(command_return::failure(ErrorCode::Fail)),
    });

    assert!(LowLevelDebug::<fake::Syscalls>::print_1(72).is_failure());

    // The fake driver still receives the command even if a fake error is injected.
    assert_eq!(driver.take_messages(), [fake::Message::Print1(72)]);
}
