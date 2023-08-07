use crate::fake::{self, SyscallDriver};
use fake::adc::*;
use libtock_platform::{share, DefaultConfig, YieldNoWaitReturn};

//Test the command implementation
#[test]
fn command() {
    let adc = Adc::new();

    assert!(adc.command(EXISTS, 1, 2).is_success_u32());

    assert!(adc.command(SINGLE_SAMPLE, 0, 0).is_success());

    assert_eq!(
        adc.command(SINGLE_SAMPLE, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    adc.set_value(100);
    assert!(adc.command(SINGLE_SAMPLE, 0, 1).is_success());
    adc.set_value(100);

    adc.set_value_sync(100);
    assert!(adc.command(SINGLE_SAMPLE, 0, 1).is_success());
    assert!(adc.command(SINGLE_SAMPLE, 0, 1).is_success());
}

// Integration test that verifies Adc works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let adc = Adc::new();
    kernel.add_driver(&adc);
    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2).is_success_u32());
    assert!(fake::Syscalls::command(DRIVER_NUM, SINGLE_SAMPLE, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, SINGLE_SAMPLE, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    adc.set_value(100);
    assert!(fake::Syscalls::command(DRIVER_NUM, SINGLE_SAMPLE, 0, 1).is_success());

    let listener = Cell::<Option<(u32,)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        adc.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        adc.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, SINGLE_SAMPLE, 0, 1).is_success());
        adc.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        adc.set_value_sync(200);
        assert!(fake::Syscalls::command(DRIVER_NUM, SINGLE_SAMPLE, 0, 1).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
}
