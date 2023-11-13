use super::*;
use core::fmt::Write;
use libtock_platform::ErrorCode;
use libtock_unittest::{command_return, fake, ExpectedSyscall};

type Console = super::Console<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert!(!Console::exists());
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);

    assert!(Console::exists());
    assert_eq!(driver.take_bytes(), &[]);
}

#[test]
fn write_bytes() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);

    Console::write(b"foo").unwrap();
    Console::write(b"bar").unwrap();
    assert_eq!(driver.take_bytes(), b"foobar",);
}

#[test]
fn write_str() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);

    write!(Console::writer(), "foo").unwrap();
    assert_eq!(driver.take_bytes(), b"foo");
}

#[test]
fn read_bytes_short() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new_with_input(b"Hello");
    kernel.add_driver(&driver);

    let mut buf = [0; 10];

    let (count, res) = Console::read(&mut buf);
    res.unwrap();
    assert_eq!(&buf[..count], b"Hello");
}

#[test]
fn read_bytes_alot() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new_with_input(b"Hello Alot!");
    kernel.add_driver(&driver);

    let mut buf = [0; 5];

    let (count, res) = Console::read(&mut buf);
    res.unwrap();
    assert_eq!(&buf[..count], b"Hello");

    let (count, res) = Console::read(&mut buf);
    res.unwrap();
    assert_eq!(&buf[..count], b" Alot");
}

#[test]
fn failed_print() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: DRIVER_NUM,
        buffer_num: allow_ro::WRITE,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: DRIVER_NUM,
        subscribe_num: subscribe::WRITE,
        skip_with_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: command::WRITE,
        argument0: 5,
        argument1: 0,
        override_return: Some(command_return::failure(ErrorCode::Fail)),
    });

    assert_eq!(Console::write(b"abcde"), Err(ErrorCode::Fail));
    // The fake driver still receives the command even if a fake error is injected.
    assert_eq!(driver.take_bytes(), b"abcde");
}

#[test]
fn failed_read() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new_with_input(b"bugxxxx");
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::AllowRw {
        driver_num: DRIVER_NUM,
        buffer_num: allow_rw::READ,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: DRIVER_NUM,
        subscribe_num: subscribe::READ,
        skip_with_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: command::READ,
        argument0: 3,
        argument1: 0,
        override_return: Some(command_return::failure(ErrorCode::Fail)),
    });

    let mut buf = [0; 3];

    let (count, res) = Console::read(&mut buf);
    assert_eq!(res, Err(ErrorCode::Fail));
    assert_eq!(count, 0);
}
