use core::cell::Cell;
use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

type SoundPressure = super::SoundPressure<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(SoundPressure::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::SoundPressure::new();
    kernel.add_driver(&driver);

    assert_eq!(SoundPressure::exists(), Ok(()));
}

#[test]
fn driver_busy() {
    let kernel = fake::Kernel::new();
    let driver = fake::SoundPressure::new();
    kernel.add_driver(&driver);

    assert_eq!(SoundPressure::read(), Ok(()));
    assert!(driver.is_busy());

    assert_eq!(SoundPressure::read(), Err(ErrorCode::Busy));
    assert_eq!(SoundPressure::read_sync(), Err(ErrorCode::Busy));
}

#[test]
fn read_pressure() {
    let kernel = fake::Kernel::new();
    let driver = fake::SoundPressure::new();
    kernel.add_driver(&driver);

    let pressure_cell: Cell<Option<u32>> = Cell::new(None);
    let listener = crate::SoundPressureListener(|pressure_val| {
        pressure_cell.set(Some(pressure_val));
    });

    share::scope(|subscribe| {
        assert_eq!(SoundPressure::read(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(
            SoundPressure::register_listener(&listener, subscribe),
            Ok(())
        );
        assert_eq!(SoundPressure::read(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(pressure_cell.get(), Some(100));

        SoundPressure::unregister_listener();
        assert_eq!(SoundPressure::read(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
}

#[test]
fn read_pressure_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::SoundPressure::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(100);
    assert_eq!(SoundPressure::read_sync(), Ok(100));
}
