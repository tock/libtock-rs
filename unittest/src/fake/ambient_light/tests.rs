use crate::fake::{self, SyscallDriver};
use fake::ambient_light::*;
use libtock_platform::{share, DefaultConfig, YieldNoWaitReturn};

//Test the command implementation
#[test]
fn command() {
    let amb = AmbientLight::new();

    assert!(amb.command(EXISTS, 1, 2).is_success());

    assert!(amb.command(READ_INTENSITY, 0, 0).is_success());

    assert_eq!(
        amb.command(READ_INTENSITY, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    amb.set_value(100);
    assert!(amb.command(READ_INTENSITY, 0, 1).is_success());
    amb.set_value(100);

    amb.set_value_sync(100);
    assert!(amb.command(READ_INTENSITY, 0, 1).is_success());
    assert!(amb.command(READ_INTENSITY, 0, 1).is_success());
}

// Integration test that verifies AmbientLight works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let ambient_light = AmbientLight::new();
    kernel.add_driver(&ambient_light);
    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_INTENSITY, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, READ_INTENSITY, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    ambient_light.set_value(100);
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_INTENSITY, 0, 1).is_success());

    let listener = Cell::<Option<(u32,)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        ambient_light.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        ambient_light.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, READ_INTENSITY, 0, 1).is_success());
        ambient_light.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        ambient_light.set_value_sync(200);
        assert!(fake::Syscalls::command(DRIVER_NUM, READ_INTENSITY, 0, 1).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
}
