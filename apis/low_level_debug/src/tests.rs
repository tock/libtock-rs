use super::*;
use libtock_platform::ErrorCode;
use libtock_unittest::{command_return, fake, ExpectedSyscall};

type LowLevelDebug = super::LowLevelDebug<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert!(!LowLevelDebug::exists());
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);

    assert!(LowLevelDebug::exists());
    assert_eq!(driver.take_messages(), []);
}

#[test]
fn print_alert_code() {
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);

    LowLevelDebug::print_alert_code(AlertCode::Panic);
    LowLevelDebug::print_alert_code(AlertCode::WrongLocation);
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
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);

    LowLevelDebug::print_1(42);
    assert_eq!(driver.take_messages(), [fake::Message::Print1(42)]);
}

#[test]
fn print_2() {
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);

    LowLevelDebug::print_2(42, 27);
    LowLevelDebug::print_2(29, 43);
    assert_eq!(
        driver.take_messages(),
        [fake::Message::Print2(42, 27), fake::Message::Print2(29, 43)]
    );
}

#[test]
fn failed_print() {
    let kernel = fake::Kernel::new();
    let driver = fake::LowLevelDebug::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: PRINT_1,
        argument0: 72,
        argument1: 0,
        override_return: Some(command_return::failure(ErrorCode::Fail)),
    });

    // The error is explicitly silenced, and cannot be detected.
    LowLevelDebug::print_1(72);

    // The fake driver still receives the command even if a fake error is injected.
    assert_eq!(driver.take_messages(), [fake::Message::Print1(72)]);
}
