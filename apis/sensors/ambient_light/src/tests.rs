use core::cell::Cell;
use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

use crate::IntensityListener;

type AmbientLight = super::AmbientLight<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(AmbientLight::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::AmbientLight::new();
    kernel.add_driver(&driver);

    assert_eq!(AmbientLight::exists(), Ok(()));
}

#[test]
fn read_temperature() {
    let kernel = fake::Kernel::new();
    let driver = fake::AmbientLight::new();
    kernel.add_driver(&driver);

    assert_eq!(AmbientLight::read_intensity(), Ok(()));
    assert!(driver.is_busy());

    assert_eq!(AmbientLight::read_intensity(), Err(ErrorCode::Busy));
    assert_eq!(AmbientLight::read_intensity_sync(), Err(ErrorCode::Busy));
}

#[test]
fn register_unregister_listener() {
    let kernel = fake::Kernel::new();
    let driver = fake::AmbientLight::new();
    kernel.add_driver(&driver);

    let intensity_cell: Cell<Option<u32>> = Cell::new(None);
    let listener = IntensityListener(|val| {
        intensity_cell.set(Some(val));
    });
    share::scope(|subscribe| {
        assert_eq!(AmbientLight::read_intensity(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(
            AmbientLight::register_listener(&listener, subscribe),
            Ok(())
        );
        assert_eq!(AmbientLight::read_intensity(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(intensity_cell.get(), Some(100));

        AmbientLight::unregister_listener();
        assert_eq!(AmbientLight::read_intensity(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
}

#[test]
fn read_temperature_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::AmbientLight::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(1000);
    assert_eq!(AmbientLight::read_intensity_sync(), Ok(1000));
}
