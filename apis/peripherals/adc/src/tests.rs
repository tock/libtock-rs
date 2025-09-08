use core::cell::Cell;
use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

type Adc = super::Adc<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Adc::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Adc::new();
    kernel.add_driver(&driver);

    assert_eq!(Adc::exists(), Ok(()));
}

#[test]
fn read_single_sample() {
    let kernel = fake::Kernel::new();
    let driver = fake::Adc::new();
    kernel.add_driver(&driver);

    let ch = Adc::get_number_of_channels().unwrap();

    assert_eq!(Adc::read_single_sample(ch), Ok(()));
    assert!(driver.is_busy());

    assert_eq!(Adc::read_single_sample(ch), Err(ErrorCode::Busy));
    assert_eq!(Adc::read_single_sample_sync(ch), Err(ErrorCode::Busy));
}

#[test]
fn register_unregister_listener() {
    let kernel = fake::Kernel::new();
    let driver = fake::Adc::new();
    kernel.add_driver(&driver);

    let sample: Cell<Option<u16>> = Cell::new(None);
    let listener = crate::ADCListener(|adc_val| {
        sample.set(Some(adc_val));
    });
    share::scope(|subscribe| {
        let ch = Adc::get_number_of_channels().unwrap();

        assert_eq!(Adc::read_single_sample(ch), Ok(()));
        driver.set_value(ch.try_into().unwrap(), 100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(Adc::register_listener(&listener, subscribe), Ok(()));
        assert_eq!(Adc::read_single_sample(ch), Ok(()));
        driver.set_value(ch.try_into().unwrap(), 100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(sample.get(), Some(100));

        Adc::unregister_listener();
        assert_eq!(Adc::read_single_sample(ch), Ok(()));
        driver.set_value(ch.try_into().unwrap(), 100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
}

#[test]
fn read_single_sample_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::Adc::new();
    kernel.add_driver(&driver);

    let ch = Adc::get_number_of_channels().unwrap();

    driver.set_value_sync(ch.try_into().unwrap(), 1000);
    assert_eq!(Adc::read_single_sample_sync(ch), Ok(1000));
}
