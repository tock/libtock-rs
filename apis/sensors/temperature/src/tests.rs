use core::cell::Cell;
use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

type Temperature = super::Temperature<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Temperature::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Temperature::new();
    kernel.add_driver(&driver);

    assert_eq!(Temperature::exists(), Ok(()));
}

#[test]
fn read_temperature() {
    let kernel = fake::Kernel::new();
    let driver = fake::Temperature::new();
    kernel.add_driver(&driver);

    assert_eq!(Temperature::read_temperature(), Ok(()));
    assert!(driver.is_busy());

    assert_eq!(Temperature::read_temperature(), Err(ErrorCode::Busy));
    assert_eq!(Temperature::read_temperature_sync(), Err(ErrorCode::Busy));
}

#[test]
fn register_unregister_listener() {
    let kernel = fake::Kernel::new();
    let driver = fake::Temperature::new();
    kernel.add_driver(&driver);

    let temperature_cell: Cell<Option<i32>> = Cell::new(None);
    let listener = crate::TemperatureListener(|temp_val| {
        temperature_cell.set(Some(temp_val));
    });
    share::scope(|subscribe| {
        assert_eq!(Temperature::read_temperature(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(Temperature::register_listener(&listener, subscribe), Ok(()));
        assert_eq!(Temperature::read_temperature(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(temperature_cell.get(), Some(100));

        Temperature::unregister_listener();
        assert_eq!(Temperature::read_temperature(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
}

#[test]
fn read_temperature_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::Temperature::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(1000);
    assert_eq!(Temperature::read_temperature_sync(), Ok(1000));
}

#[test]
fn negative_value() {
    let kernel = fake::Kernel::new();
    let driver = fake::Temperature::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(-1000);
    assert_eq!(Temperature::read_temperature_sync(), Ok(-1000));
}
