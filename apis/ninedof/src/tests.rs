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
fn driver_check() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    assert_eq!(NineDof::exists(), Ok(()));
}

#[test]
fn read_acceleration() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    assert_eq!(NineDof::read_accelerometer(), Ok(()));
    assert!(driver.is_busy());

    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut z: i32 = 0;
    assert_eq!(NineDof::read_accelerometer(), Err(ErrorCode::Busy));
    assert_eq!(
        NineDof::read_accelerometer_sync(&mut x, &mut y, &mut z),
        Err(ErrorCode::Busy)
    );
}

#[test]
fn read_magnetometer() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    assert_eq!(NineDof::read_magnetometer(), Ok(()));
    assert!(driver.is_busy());

    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut z: i32 = 0;
    assert_eq!(NineDof::read_magnetometer(), Err(ErrorCode::Busy));
    assert_eq!(
        NineDof::read_magnetometer_sync(&mut x, &mut y, &mut z),
        Err(ErrorCode::Busy)
    );
}

#[test]
fn read_gyro() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    assert_eq!(NineDof::read_gyro(), Ok(()));
    assert!(driver.is_busy());

    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut z: i32 = 0;
    assert_eq!(NineDof::read_gyro(), Err(ErrorCode::Busy));
    assert_eq!(
        NineDof::read_gyro_sync(&mut x, &mut y, &mut z),
        Err(ErrorCode::Busy)
    );
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

    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut z: i32 = 0;

    driver.set_value_sync(fake::NineDofData { x: 1, y: 2, z: 3 });

    assert_eq!(
        NineDof::read_accelerometer_sync(&mut x, &mut y, &mut z),
        Ok(())
    );
    assert_eq!(x, 1);
    assert_eq!(y, 2);
    assert_eq!(z, 3);
}

#[test]
fn read_magnetometer_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut z: i32 = 0;

    driver.set_value_sync(fake::NineDofData { x: 1, y: 2, z: 3 });

    assert_eq!(
        NineDof::read_magnetometer_sync(&mut x, &mut y, &mut z),
        Ok(())
    );
    assert_eq!(x, 1);
    assert_eq!(y, 2);
    assert_eq!(z, 3);
}

#[test]
fn read_gyro_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::NineDof::new();
    kernel.add_driver(&driver);

    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut z: i32 = 0;

    driver.set_value_sync(fake::NineDofData { x: 1, y: 2, z: 3 });

    assert_eq!(NineDof::read_gyro_sync(&mut x, &mut y, &mut z), Ok(()));
    assert_eq!(x, 1);
    assert_eq!(y, 2);
    assert_eq!(z, 3);
}
