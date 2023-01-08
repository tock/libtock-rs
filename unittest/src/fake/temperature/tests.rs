use crate::fake::{self, SyscallDriver};
use fake::temperature::*;
use libtock_platform::{share, DefaultConfig, YieldNoWaitReturn};

//Test the command implementation
#[test]
fn command() {
    let temp = Temperature::new();

    assert!(temp.command(EXISTS, 1, 2).is_success());

    assert!(temp.command(READ_TEMP, 0, 0).is_success());

    assert_eq!(
        temp.command(READ_TEMP, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    temp.set_value(100);
    assert!(temp.command(READ_TEMP, 0, 1).is_success());
    temp.set_value(100);

    temp.set_value_sync(100);
    assert!(temp.command(READ_TEMP, 0, 1).is_success());
    assert!(temp.command(READ_TEMP, 0, 1).is_success());
}

// Integration test that verifies Temperature works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let temp = Temperature::new();
    kernel.add_driver(&temp);
    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_TEMP, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, READ_TEMP, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    temp.set_value(100);
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_TEMP, 0, 1).is_success());

    let listener = Cell::<Option<(u32,)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        temp.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        temp.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, READ_TEMP, 0, 1).is_success());
        temp.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        temp.set_value_sync(200);
        assert!(fake::Syscalls::command(DRIVER_NUM, READ_TEMP, 0, 1).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
}
