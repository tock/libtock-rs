use core::cell::Cell;
use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

type Proximity = super::Proximity<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Proximity::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Proximity::new();
    kernel.add_driver(&driver);

    assert_eq!(Proximity::exists(), Ok(()));
}

#[test]
fn busy_driver() {
    let kernel = fake::Kernel::new();
    let driver = fake::Proximity::new();
    kernel.add_driver(&driver);

    assert_eq!(Proximity::read(), Ok(()));
    assert_eq!(Proximity::read(), Err(ErrorCode::Busy));
    assert_eq!(Proximity::read_on_interrupt(0, 0), Err(ErrorCode::Busy));

    driver.set_value(100);

    assert_eq!(Proximity::read_on_interrupt(0, 0), Ok(()));
    assert_eq!(Proximity::read(), Err(ErrorCode::Busy));
}

#[test]
fn async_readings() {
    let kernel = fake::Kernel::new();
    let driver = fake::Proximity::new();
    kernel.add_driver(&driver);

    let listener = Cell::<Option<(u32,)>>::new(None);

    share::scope(|subscribe| {
        assert_eq!(Proximity::read(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(Proximity::register_listener(&listener, subscribe), Ok(()));
        assert_eq!(Proximity::read(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        assert_eq!(Proximity::read_on_interrupt(100, 200), Ok(()));
        driver.set_value(150);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        driver.set_value(99);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((99,)));
    })
}

#[test]
fn sync_readings() {
    let kernel = fake::Kernel::new();
    let driver = fake::Proximity::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(100);
    assert_eq!(Proximity::read_sync(), Ok(100));

    driver.set_value_sync(250);
    assert_eq!(Proximity::wait_for_value_between(100, 200), Ok(250));
}

#[test]
fn bad_arguments() {
    assert_eq!(
        Proximity::wait_for_value_between(200, 100),
        Err(ErrorCode::Invalid)
    );
}
