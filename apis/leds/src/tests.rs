use libtock_platform::ErrorCode;
use libtock_unittest::fake;

type Leds = super::Leds<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Leds::count(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    assert_eq!(Leds::count(), Ok(10));
    for led in 0..10 {
        assert_eq!(driver.get_led(led), Some(false));
    }
}

#[test]
fn num_leds() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);
    assert_eq!(Leds::count().unwrap_or_default(), 10);
}

#[test]
fn on() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    assert_eq!(Leds::on(0), Ok(()));
    assert_eq!(driver.get_led(0), Some(true));
}

#[test]
fn off() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    assert_eq!(Leds::off(0), Ok(()));
    assert_eq!(driver.get_led(0), Some(false));
}

#[test]
fn toggle() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    assert_eq!(Leds::toggle(0), Ok(()));
    assert_eq!(driver.get_led(0), Some(true));
    assert_eq!(Leds::toggle(0), Ok(()));
    assert_eq!(driver.get_led(0), Some(false));
}

#[test]
fn on_off() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    assert_eq!(Leds::on(0), Ok(()));
    assert_eq!(driver.get_led(0), Some(true));
    assert_eq!(Leds::off(0), Ok(()));
    assert_eq!(driver.get_led(0), Some(false));
}

#[test]
fn no_led() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    assert_eq!(Leds::on(11), Err(ErrorCode::Invalid));
    for led in 0..Leds::count().unwrap_or_default() {
        assert_eq!(driver.get_led(led), Some(false));
    }
}
