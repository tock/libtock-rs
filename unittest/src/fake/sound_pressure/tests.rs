use crate::fake::{self, SyscallDriver};
use fake::sound_pressure::*;
use libtock_platform::{share, DefaultConfig, YieldNoWaitReturn};

//Test the command implementation
#[test]
fn command() {
    let sound_pressure = SoundPressure::new();

    assert!(sound_pressure.command(EXISTS, 1, 2).is_success());

    assert!(sound_pressure.command(READ_PRESSURE, 0, 0).is_success());

    assert_eq!(
        sound_pressure.command(READ_PRESSURE, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    sound_pressure.set_value(100);
    assert!(sound_pressure.command(READ_PRESSURE, 0, 1).is_success());
    sound_pressure.set_value(100);

    sound_pressure.set_value_sync(100);
    assert!(sound_pressure.command(READ_PRESSURE, 0, 1).is_success());
    assert!(sound_pressure.command(READ_PRESSURE, 0, 1).is_success());
}

// Integration test that verifies SoundPressure works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let sound_pressure = SoundPressure::new();
    kernel.add_driver(&sound_pressure);
    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_PRESSURE, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, READ_PRESSURE, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    sound_pressure.set_value(100);
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_PRESSURE, 0, 1).is_success());

    let listener = Cell::<Option<(u32,)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        sound_pressure.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        sound_pressure.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, READ_PRESSURE, 0, 1).is_success());
        sound_pressure.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((200,)));
    });
}
