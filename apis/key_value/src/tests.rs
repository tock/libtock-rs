use super::*;
use libtock_platform::ErrorCode;
use libtock_unittest::{command_return, fake, ExpectedSyscall};

type Kv = super::KeyValue<fake::Syscalls>;

fn _get(kernel: &fake::Kernel) -> Result<u32, ErrorCode> {
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: DRIVER_NUM,
        buffer_num: allow_ro::KEY,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::AllowRw {
        driver_num: DRIVER_NUM,
        buffer_num: allow_rw::VALUE_READ,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: DRIVER_NUM,
        subscribe_num: subscribe::CALLBACK,
        skip_with_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: command::GET,
        argument0: 0,
        argument1: 0,
        override_return: Some(command_return::success()),
    });

    let mut buf = [0; 3];
    Kv::get("mykey".as_bytes(), &mut buf)
}

fn _set(kernel: &fake::Kernel) -> Result<(), ErrorCode> {
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: DRIVER_NUM,
        buffer_num: allow_ro::KEY,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: DRIVER_NUM,
        buffer_num: allow_ro::VALUE_WRITE,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: DRIVER_NUM,
        subscribe_num: subscribe::CALLBACK,
        skip_with_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: command::SET,
        argument0: 0,
        argument1: 0,
        override_return: Some(command_return::success()),
    });

    Kv::set("mykey".as_bytes(), b"hooray")
}

fn _add(kernel: &fake::Kernel) -> Result<(), ErrorCode> {
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: DRIVER_NUM,
        buffer_num: allow_ro::KEY,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: DRIVER_NUM,
        buffer_num: allow_ro::VALUE_WRITE,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: DRIVER_NUM,
        subscribe_num: subscribe::CALLBACK,
        skip_with_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: command::ADD,
        argument0: 0,
        argument1: 0,
        override_return: Some(command_return::success()),
    });

    Kv::add("mykey".as_bytes(), b"hooray2")
}

fn _update(kernel: &fake::Kernel) -> Result<(), ErrorCode> {
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: DRIVER_NUM,
        buffer_num: allow_ro::KEY,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: DRIVER_NUM,
        buffer_num: allow_ro::VALUE_WRITE,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: DRIVER_NUM,
        subscribe_num: subscribe::CALLBACK,
        skip_with_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: command::UPDATE,
        argument0: 0,
        argument1: 0,
        override_return: Some(command_return::success()),
    });

    Kv::update("mykey".as_bytes(), b"hooray3")
}

fn _delete(kernel: &fake::Kernel) -> Result<(), ErrorCode> {
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: DRIVER_NUM,
        buffer_num: allow_ro::KEY,
        return_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: DRIVER_NUM,
        subscribe_num: subscribe::CALLBACK,
        skip_with_error: None,
    });
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: command::DELETE,
        argument0: 0,
        argument1: 0,
        override_return: Some(command_return::success()),
    });

    Kv::delete("mykey".as_bytes())
}

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert!(!Kv::exists());
}

#[test]
fn driver_check() {
    let kernel = fake::Kernel::new();
    let driver = fake::KeyValue::new();
    kernel.add_driver(&driver);

    assert!(Kv::exists());
}

#[test]
fn get_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::KeyValue::new();
    kernel.add_driver(&driver);

    assert_eq!(_get(&kernel), Err(ErrorCode::NoSupport));
}

#[test]
fn set_get() {
    let kernel = fake::Kernel::new();
    let driver = fake::KeyValue::new();
    kernel.add_driver(&driver);

    assert_eq!(_set(&kernel), Ok(()));
    assert_eq!(_get(&kernel), Ok(6));
}

#[test]
fn add() {
    let kernel = fake::Kernel::new();
    let driver = fake::KeyValue::new();
    kernel.add_driver(&driver);

    assert_eq!(_add(&kernel), Ok(()));
}

#[test]
fn add_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::KeyValue::new();
    kernel.add_driver(&driver);

    assert_eq!(_set(&kernel), Ok(()));
    assert_eq!(_add(&kernel), Err(ErrorCode::NoSupport));
}

#[test]
fn update() {
    let kernel = fake::Kernel::new();
    let driver = fake::KeyValue::new();
    kernel.add_driver(&driver);

    assert_eq!(_set(&kernel), Ok(()));
    assert_eq!(_update(&kernel), Ok(()));
}

#[test]
fn update_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::KeyValue::new();
    kernel.add_driver(&driver);

    assert_eq!(_update(&kernel), Err(ErrorCode::NoSupport));
}

#[test]
fn delete() {
    let kernel = fake::Kernel::new();
    let driver = fake::KeyValue::new();
    kernel.add_driver(&driver);

    assert_eq!(_set(&kernel), Ok(()));
    assert_eq!(_delete(&kernel), Ok(()));
}

#[test]
fn delete_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::KeyValue::new();
    kernel.add_driver(&driver);

    assert_eq!(_delete(&kernel), Err(ErrorCode::NoSupport));
}
