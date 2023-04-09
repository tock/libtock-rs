use crate::fake::{self, SyscallDriver};
use fake::proximity::*;
use libtock_platform::{share, DefaultConfig, YieldNoWaitReturn};

//Test the command implementation
#[test]
fn command() {
    let proximity = Proximity::new();

    assert!(proximity.command(EXISTS, 0, 0).is_success());

    assert!(proximity.command(READ, 0, 0).is_success());

    assert_eq!(
        proximity.command(READ, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    proximity.set_value(0);
    assert!(proximity.command(READ_ON_INT, 10, 20).is_success());

    //should still be busy, value does not meet conditions
    proximity.set_value(15);
    assert_eq!(
        proximity.command(READ, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    proximity.set_value(50);
    assert!(proximity.command(READ, 0, 0).is_success());
    proximity.set_value(50);

    proximity.set_value_sync(10);
    assert!(proximity.command(READ, 0, 0).is_success());

    //upcall was called after command, should not be busy

    assert!(proximity.command(READ, 0, 0).is_success());
}

// Integration test that verifies Proximity works with fake::Kernel and
// libtock_platform::Syscalls.

#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let proximity = Proximity::new();
    kernel.add_driver(&proximity);
    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 0, 0).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, READ, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, READ, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    proximity.set_value(100);

    let listener = Cell::<Option<(u32,)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        //should not call an upcall as no command was given
        proximity.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, READ, 0, 0).is_success());
        //now it should call an upcall
        proximity.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        assert!(fake::Syscalls::command(DRIVER_NUM, READ_ON_INT, 50, 150).is_success());
        proximity.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        proximity.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        proximity.set_value_sync(100);
        assert!(fake::Syscalls::command(DRIVER_NUM, READ, 0, 0).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    })
}
