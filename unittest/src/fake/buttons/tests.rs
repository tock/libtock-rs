use crate::fake;
use fake::buttons::*;
use libtock_platform::ErrorCode;

// Tests the command implementation.
#[test]
fn command() {
    use fake::SyscallDriver;
    let buttons = Buttons::<10>::new();

    assert_eq!(
        buttons.command(BUTTONS_COUNT, 1, 2).get_success_u32(),
        Some(10)
    );

    assert_eq!(
        buttons.command(BUTTONS_READ, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        buttons
            .command(BUTTONS_ENABLE_INTERRUPTS, 11, 0)
            .get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        buttons
            .command(BUTTONS_DISABLE_INTERRUPTS, 11, 0)
            .get_failure(),
        Some(ErrorCode::Invalid)
    );

    for button_index in 0..10 {
        assert_eq!(
            buttons.get_button_state(button_index),
            Some(ButtonState {
                pressed: false,
                interrupt_enabled: false
            })
        );

        assert!(buttons
            .command(BUTTONS_ENABLE_INTERRUPTS, button_index, 0)
            .is_success());
        assert_eq!(
            buttons.get_button_state(button_index),
            Some(ButtonState {
                pressed: false,
                interrupt_enabled: true
            })
        );

        assert!(buttons
            .command(BUTTONS_DISABLE_INTERRUPTS, button_index, 0)
            .is_success());
        assert_eq!(
            buttons.get_button_state(button_index),
            Some(ButtonState {
                pressed: false,
                interrupt_enabled: false
            })
        );

        assert_eq!(buttons.set_pressed(button_index, true), Ok(()));
        assert_eq!(
            buttons.get_button_state(button_index),
            Some(ButtonState {
                pressed: true,
                interrupt_enabled: false
            })
        );

        assert_eq!(buttons.set_pressed(button_index, false), Ok(()));
        assert_eq!(
            buttons.get_button_state(button_index),
            Some(ButtonState {
                pressed: false,
                interrupt_enabled: false
            })
        );
    }
}

// Integration test that verifies Buttons works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let buttons = Buttons::<10>::new();
    kernel.add_driver(&buttons);
    let value = fake::Syscalls::command(DRIVER_NUM, BUTTONS_COUNT, 1, 2);
    assert_eq!(value.get_success_u32(), Some(10));

    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, BUTTONS_READ, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert!(fake::Syscalls::command(DRIVER_NUM, BUTTONS_READ, 0, 0).is_success_u32());

    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, BUTTONS_ENABLE_INTERRUPTS, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert!(fake::Syscalls::command(DRIVER_NUM, BUTTONS_ENABLE_INTERRUPTS, 0, 0).is_success());

    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, BUTTONS_DISABLE_INTERRUPTS, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert!(fake::Syscalls::command(DRIVER_NUM, BUTTONS_DISABLE_INTERRUPTS, 0, 0).is_success());
}
