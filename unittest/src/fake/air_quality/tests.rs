use crate::fake::{self, SyscallDriver};
use fake::air_quality::*;
use libtock_platform::{share::scope, DefaultConfig, YieldNoWaitReturn};

//Test the `command` implementation
#[test]
fn command() {
    let driver = AirQuality::new();

    assert!(driver.command(EXISTS, 0, 0).is_success());

    driver.set_co2_available(false);
    assert_eq!(
        driver.command(READ_CO2, 0, 0).get_failure(),
        Some(ErrorCode::NoSupport)
    );
    driver.set_tvoc_available(false);
    assert_eq!(
        driver.command(READ_TVOC, 0, 0).get_failure(),
        Some(ErrorCode::NoSupport)
    );

    driver.set_co2_available(true);
    driver.set_tvoc_available(true);

    assert!(driver.command(READ_CO2, 0, 0).is_success());
    assert_eq!(
        driver.command(READ_CO2, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    driver.set_value(100);
    assert!(driver.command(READ_CO2, 0, 0).is_success());
    driver.set_value(100);

    assert!(driver.command(READ_TVOC, 0, 0).is_success());
    assert_eq!(
        driver.command(READ_TVOC, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    driver.set_value(100);
    assert!(driver.command(READ_TVOC, 0, 0).is_success());
    driver.set_value(100);

    driver.set_value_sync(100);
    assert!(driver.command(READ_CO2, 0, 0).is_success());
    assert!(driver.command(READ_TVOC, 0, 0).is_success());
}

// Integration test that verifies Temperature works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let driver = AirQuality::new();
    kernel.add_driver(&driver);

    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 0, 0).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_CO2, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, READ_CO2, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    driver.set_value(100);
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_CO2, 0, 0).is_success());
    driver.set_value(100);

    assert!(fake::Syscalls::command(DRIVER_NUM, READ_TVOC, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, READ_TVOC, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    driver.set_value(100);
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_TVOC, 0, 0).is_success());

    let listener = Cell::<Option<(u32,)>>::new(None);
    scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        driver.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        assert_eq!(listener.get(), Some((100,)));

        assert!(fake::Syscalls::command(DRIVER_NUM, READ_CO2, 0, 0).is_success());
        driver.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((200,)));

        driver.set_value_sync(200);
        assert!(fake::Syscalls::command(DRIVER_NUM, READ_CO2, 0, 0).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, READ_TVOC, 0, 0).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        driver.set_value_sync(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        driver.set_values_sync(100, 200);
        assert!(fake::Syscalls::command(DRIVER_NUM, READ_CO2, 0, 0).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert!(fake::Syscalls::command(DRIVER_NUM, READ_TVOC, 0, 0).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert!(fake::Syscalls::command(DRIVER_NUM, READ_CO2, 0, 0).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
}
