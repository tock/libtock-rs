use crate::AirQualityListener;
use core::cell::Cell;
use libtock_platform::{share::scope, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

type AirQuality = super::AirQuality<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(AirQuality::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::AirQuality::new();
    kernel.add_driver(&driver);

    assert_eq!(AirQuality::exists(), Ok(()));
}

#[test]
fn read_co2() {
    let kernel = fake::Kernel::new();
    let driver = fake::AirQuality::new();
    kernel.add_driver(&driver);

    assert_eq!(AirQuality::read_co2(), Ok(()));
    assert!(driver.is_busy());

    assert_eq!(AirQuality::read_co2(), Err(ErrorCode::Busy));
    assert_eq!(AirQuality::read_co2_sync(), Err(ErrorCode::Busy));
}

#[test]
fn read_tvoc() {
    let kernel = fake::Kernel::new();
    let driver = fake::AirQuality::new();
    kernel.add_driver(&driver);

    assert_eq!(AirQuality::read_tvoc(), Ok(()));
    assert!(driver.is_busy());

    assert_eq!(AirQuality::read_tvoc(), Err(ErrorCode::Busy));
    assert_eq!(AirQuality::read_tvoc_sync(), Err(ErrorCode::Busy));
}

#[test]
fn register_unregister_listener() {
    let kernel = fake::Kernel::new();
    let driver = fake::AirQuality::new();
    kernel.add_driver(&driver);

    let data_cell: Cell<Option<u32>> = Cell::new(None);
    let listener = AirQualityListener(|data_val| {
        data_cell.set(Some(data_val));
    });

    scope(|subscribe| {
        assert_eq!(AirQuality::read_co2(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(AirQuality::read_tvoc(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(AirQuality::register_listener(&listener, subscribe), Ok(()));

        assert_eq!(AirQuality::read_co2(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(data_cell.get(), Some(100));

        assert_eq!(AirQuality::read_tvoc(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(data_cell.get(), Some(100));

        AirQuality::unregister_listener();
        assert_eq!(AirQuality::read_co2(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(AirQuality::read_tvoc(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
}

#[test]
fn read_co2_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::AirQuality::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(100);
    assert_eq!(AirQuality::read_co2_sync(), Ok(100));
}

#[test]
fn read_tvoc_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::AirQuality::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(100);
    assert_eq!(AirQuality::read_tvoc_sync(), Ok(100));
}

#[test]
fn read_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::AirQuality::new();
    kernel.add_driver(&driver);

    driver.set_values_sync(100, 200);
    assert_eq!(AirQuality::read_sync(), Ok((100, 200)))
}
