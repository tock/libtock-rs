use core::cell::Cell;
use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

use crate::NineDofData;

type NineDof = super::NineDof<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(NineDof::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    assert_eq!(NineDof::exists(), Ok(()));
}

#[test]
fn driver_busy() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    assert_eq!(NineDof::read_accelerometer(), Ok(()));
    assert!(driver.is_busy());

    assert_eq!(NineDof::read_accelerometer(), Err(ErrorCode::Busy));
    assert_eq!(NineDof::read_accelerometer_sync(), Err(ErrorCode::Busy));
}

#[test]
fn read_accelerometer() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    let acceleration_listener: Cell<Option<NineDofData>> = Cell::new(None);
    let acceleration_listener = crate::NineDofListener(|data| {
        acceleration_listener.set(Some(data));
    });

    share::scope(|subscribe| {
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        assert_eq!(
            NineDof::register_listener(&acceleration_listener, subscribe),
            Ok(())
        );
        assert_eq!(NineDof::read_accelerometer(), Ok(()));
        driver.set_value(fake::NineDofData { x: 1, y: 2, z: 3 });
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
}

#[test]
fn read_magnetometer() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    let magnetometer_listener: Cell<Option<NineDofData>> = Cell::new(None);
    let magnetometer_listener = crate::NineDofListener(|data| {
        magnetometer_listener.set(Some(data));
    });

    share::scope(|subscribe| {
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        assert_eq!(
            NineDof::register_listener(&magnetometer_listener, subscribe),
            Ok(())
        );
        assert_eq!(NineDof::read_accelerometer(), Ok(()));
        driver.set_value(fake::NineDofData { x: 1, y: 2, z: 3 });
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
}

#[test]
fn read_gyro() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    let gyro_listener: Cell<Option<NineDofData>> = Cell::new(None);
    let gyro_listener = crate::NineDofListener(|data| {
        gyro_listener.set(Some(data));
    });

    share::scope(|subscribe| {
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        assert_eq!(
            NineDof::register_listener(&gyro_listener, subscribe),
            Ok(())
        );
        assert_eq!(NineDof::read_accelerometer(), Ok(()));
        driver.set_value(fake::NineDofData { x: 1, y: 2, z: 3 });
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
}

#[test]
fn register_unregister_listener() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    let acceleration_listener: Cell<Option<NineDofData>> = Cell::new(None);
    let acceleration_listener = crate::NineDofListener(|data| {
        acceleration_listener.set(Some(data));
    });
    let magnetometer_listener: Cell<Option<NineDofData>> = Cell::new(None);
    let magnetometer_listener = crate::NineDofListener(|data| {
        magnetometer_listener.set(Some(data));
    });
    let gyro_listener: Cell<Option<NineDofData>> = Cell::new(None);
    let gyro_listener = crate::NineDofListener(|data| {
        gyro_listener.set(Some(data));
    });

    share::scope(|subscribe| {
        assert_eq!(NineDof::read_accelerometer(), Ok(()));
        driver.set_value(fake::NineDofData { x: 1, y: 2, z: 3 });
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(
            NineDof::register_listener(&acceleration_listener, subscribe),
            Ok(())
        );

        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        assert_eq!(NineDof::read_gyro(), Ok(()));
        driver.set_value(fake::NineDofData { x: 4, y: 5, z: 6 });
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        assert_eq!(
            NineDof::register_listener(&gyro_listener, subscribe),
            Ok(())
        );

        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        assert_eq!(NineDof::read_magnetometer(), Ok(()));
        driver.set_value(fake::NineDofData { x: 7, y: 8, z: 9 });
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        assert_eq!(
            NineDof::register_listener(&magnetometer_listener, subscribe),
            Ok(())
        );
    })
}

#[test]
fn read_accelerometer_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(fake::NineDofData { x: 1, y: 2, z: 3 });
    let data = NineDof::read_accelerometer_sync();
    assert_eq!(data, Ok(NineDofData { x: 1, y: 2, z: 3 }));
}

#[test]
fn read_magnetometer_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(fake::NineDofData { x: 1, y: 2, z: 3 });

    let data = NineDof::read_magnetometer_sync();

    assert_eq!(data, Ok(NineDofData { x: 1, y: 2, z: 3 }));
}

#[test]
fn read_gyro_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(fake::NineDofData { x: 1, y: 2, z: 3 });
    let value = NineDof::read_gyroscope_sync();
    assert_eq!(value, Ok(NineDofData { x: 1, y: 2, z: 3 }));
}
