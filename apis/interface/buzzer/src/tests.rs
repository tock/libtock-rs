use core::time::Duration;
use libtock_platform::ErrorCode;
use libtock_unittest::fake;

type Buzzer = super::Buzzer<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Buzzer::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Buzzer::new();
    kernel.add_driver(&driver);

    assert_eq!(Buzzer::exists(), Ok(()));
}

#[test]
fn tone() {
    let kernel = fake::Kernel::new();
    let driver = fake::Buzzer::new();
    kernel.add_driver(&driver);
    let duration = Duration::from_millis(100);
    assert_eq!(Buzzer::tone(1000, duration), Ok(()));
    assert!(driver.is_busy());

    assert_eq!(Buzzer::tone(1000, duration), Err(ErrorCode::Busy));
}

#[test]
fn tone_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::Buzzer::new();
    kernel.add_driver(&driver);

    let duration = Duration::from_millis(100);

    driver.set_tone_sync(1000, 100);
    assert_eq!(Buzzer::tone_sync(1000, duration), Ok(()));
}
