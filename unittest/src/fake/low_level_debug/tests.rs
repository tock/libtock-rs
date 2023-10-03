use crate::fake;
use fake::low_level_debug::*;

// Tests the command implementation.
#[test]
fn command() {
    use fake::SyscallDriver;
    let low_level_debug = LowLevelDebug::new();
    assert!(low_level_debug.command(EXISTS, 1, 2).is_success());
    assert!(low_level_debug.command(PRINT_ALERT_CODE, 3, 4).is_success());
    assert_eq!(low_level_debug.take_messages(), [Message::AlertCode(3)]);
    assert!(low_level_debug.command(PRINT_1, 5, 6).is_success());
    assert!(low_level_debug.command(PRINT_2, 7, 8).is_success());
    assert_eq!(
        low_level_debug.take_messages(),
        [Message::Print1(5), Message::Print2(7, 8)]
    );
}

// Integration test that verifies LowLevelDebug works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let low_level_debug = LowLevelDebug::new();
    kernel.add_driver(&low_level_debug);
    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, PRINT_ALERT_CODE, 3, 4).is_success());
    assert_eq!(low_level_debug.take_messages(), [Message::AlertCode(3)]);
    assert!(fake::Syscalls::command(DRIVER_NUM, PRINT_1, 5, 6).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, PRINT_2, 7, 8).is_success());
    assert_eq!(
        low_level_debug.take_messages(),
        [Message::Print1(5), Message::Print2(7, 8)]
    );
}

// Tests the Display implementation on Message.
#[test]
fn message_display() {
    use Message::*;
    assert_eq!(format!("{}", AlertCode(0x0)), "alert code 0x0 (unknown)");
    assert_eq!(format!("{}", AlertCode(0x1)), "alert code 0x1 (panic)");
    assert_eq!(
        format!("{}", AlertCode(0x2)),
        "alert code 0x2 (wrong location)"
    );
    assert_eq!(format!("{}", AlertCode(0x3)), "alert code 0x3 (unknown)");
    assert_eq!(format!("{}", Print1(0x31)), "prints 0x31");
    assert_eq!(format!("{}", Print2(0x41, 0x59)), "prints 0x41 0x59");
}
