use crate::fake::{self, SyscallDriver};
use fake::ninedof::*;
use libtock_platform::{share, DefaultConfig, YieldNoWaitReturn};

//Test the command implementation
#[test]
fn command() {
    let ninedof = NineDof::new();

    assert!(ninedof.command(EXISTS, 1, 2).is_success());

    assert!(ninedof.command(READ_ACCELEROMETER, 0, 0).is_success());

    assert_eq!(
        ninedof.command(READ_ACCELEROMETER, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    let payload: NineDofData = NineDofData { x: 1, y: 2, z: 3 };

    ninedof.set_value(payload);
    assert!(ninedof.command(READ_ACCELEROMETER, 0, 1).is_success());
    ninedof.set_value(payload);

    ninedof.set_value_sync(payload);
    assert!(ninedof.command(READ_ACCELEROMETER, 0, 1).is_success());
    assert!(ninedof.command(READ_ACCELEROMETER, 0, 1).is_success());
}

// Integration test that verifies NineDof works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let ninedof = NineDof::new();
    kernel.add_driver(&ninedof);
    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_ACCELEROMETER, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, READ_ACCELEROMETER, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    let payload: NineDofData = NineDofData { x: 1, y: 2, z: 3 };

    ninedof.set_value(payload);
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_ACCELEROMETER, 0, 1).is_success());

    let listener = Cell::<Option<(u32, u32, u32)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        ninedof.set_value(payload);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((1, 2, 3)));

        ninedof.set_value(payload);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, READ_ACCELEROMETER, 0, 1).is_success());
        ninedof.set_value(payload);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((1, 2, 3)));
    });
}
