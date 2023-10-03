use crate::fake;
use fake::leds::*;
use libtock_platform::ErrorCode;

// Tests the command implementation.
#[test]
fn command() {
    use fake::SyscallDriver;
    let leds = Leds::<10>::new();
    let value = leds.command(EXISTS, 1, 2);
    assert_eq!(value.get_success_u32(), Some(10));
    assert_eq!(
        leds.command(LED_ON, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(leds.get_led(0), Some(false));
    assert!(leds.command(LED_ON, 0, 0).is_success());
    assert_eq!(leds.get_led(0), Some(true));
    assert!(leds.command(LED_OFF, 0, 0).is_success());
    assert_eq!(leds.get_led(0), Some(false));
    assert!(leds.command(LED_TOGGLE, 0, 0).is_success());
    assert_eq!(leds.get_led(0), Some(true));
    assert!(leds.command(LED_TOGGLE, 0, 0).is_success());
    assert_eq!(leds.get_led(0), Some(false));
}

// Integration test that verifies Leds works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let leds = Leds::<10>::new();
    kernel.add_driver(&leds);
    let value = fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2);
    assert!(value.is_success_u32());
    assert_eq!(value.get_success_u32(), Some(10));
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, LED_ON, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(leds.get_led(0), Some(false));
    assert!(fake::Syscalls::command(DRIVER_NUM, LED_ON, 0, 0).is_success());
    assert_eq!(leds.get_led(0), Some(true));
    assert!(fake::Syscalls::command(DRIVER_NUM, LED_OFF, 0, 0).is_success());
    assert_eq!(leds.get_led(0), Some(false));
    assert!(fake::Syscalls::command(DRIVER_NUM, LED_TOGGLE, 0, 0).is_success());
    assert_eq!(leds.get_led(0), Some(true));
    assert!(fake::Syscalls::command(DRIVER_NUM, LED_TOGGLE, 0, 0).is_success());
    assert_eq!(leds.get_led(0), Some(false));
}
