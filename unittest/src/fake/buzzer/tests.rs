use crate::fake::{self, SyscallDriver};
use fake::buzzer::*;
use libtock_platform::{share, DefaultConfig, YieldNoWaitReturn};

#[test]
fn command() {
    let buzzer = Buzzer::new();
    assert!(buzzer.command(EXISTS, 1, 2).is_success());
    assert!(buzzer.command(TONE, 0, 0).is_success());

    assert_eq!(
        buzzer.command(TONE, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    buzzer.set_tone(100, Duration::from_millis(100));
    assert!(buzzer.command(TONE, 0, 1).is_success());
    buzzer.set_tone(100, Duration::from_millis(100));

    buzzer.set_tone_sync(100, 100);
    assert!(buzzer.command(TONE, 0, 1).is_success());
    assert!(buzzer.command(TONE, 0, 1).is_success());
}

#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let buzzer = Buzzer::new();
    kernel.add_driver(&buzzer);

    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, TONE, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, TONE, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    buzzer.set_tone(100, Duration::from_millis(100));
    assert!(fake::Syscalls::command(DRIVER_NUM, TONE, 0, 1).is_success());

    let listener = Cell::<Option<(u32,)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        buzzer.set_tone(100, Duration::from_millis(100));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        buzzer.set_tone(200, Duration::from_millis(100));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, TONE, 0, 1).is_success());
        buzzer.set_tone(200, Duration::from_millis(100));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((200,)));
    });
}
