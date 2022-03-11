use crate::fake;
use fake::gpio::*;
use libtock_platform::{share, DefaultConfig, ErrorCode, YieldNoWaitReturn};

// Tests the command implementation.
#[test]
fn command() {
    use fake::SyscallDriver;
    let gpio = Gpio::<10>::new();

    gpio.set_missing_gpio(1);

    assert_eq!(gpio.command(GPIO_COUNT, 1, 2).get_success_u32(), Some(10));

    // Enable Output
    assert_eq!(
        gpio.command(GPIO_ENABLE_OUTPUT, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        gpio.command(GPIO_ENABLE_OUTPUT, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert!(gpio.command(GPIO_ENABLE_OUTPUT, 0, 0).is_success());

    // Gpio Set
    assert_eq!(
        gpio.command(GPIO_SET, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        gpio.command(GPIO_SET, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert!(gpio.command(GPIO_SET, 0, 0).is_success(),);
    assert!(gpio.get_gpio_state(0).unwrap().value);

    // Gpio Clear
    assert_eq!(
        gpio.command(GPIO_CLEAR, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        gpio.command(GPIO_CLEAR, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert!(gpio.command(GPIO_CLEAR, 0, 0).is_success(),);
    assert!(!gpio.get_gpio_state(0).unwrap().value);

    // Gpio Toggle
    assert_eq!(
        gpio.command(GPIO_TOGGLE, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        gpio.command(GPIO_TOGGLE, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert!(gpio.command(GPIO_TOGGLE, 0, 0).is_success(),);
    assert!(gpio.get_gpio_state(0).unwrap().value);
    assert!(gpio.command(GPIO_TOGGLE, 0, 0).is_success(),);
    assert!(!gpio.get_gpio_state(0).unwrap().value);

    // Enable Input
    assert_eq!(
        gpio.command(GPIO_ENABLE_INPUT, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        gpio.command(GPIO_ENABLE_INPUT, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );

    assert_eq!(
        gpio.command(GPIO_ENABLE_INPUT, 0, 3).get_failure(),
        Some(ErrorCode::Invalid)
    );

    assert!(gpio.command(GPIO_ENABLE_INPUT, 0, 0).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().mode,
        GpioMode::Input(PullMode::PullNone)
    );

    assert!(gpio.command(GPIO_ENABLE_INPUT, 0, 1).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().mode,
        GpioMode::Input(PullMode::PullUp)
    );

    assert!(gpio.command(GPIO_ENABLE_INPUT, 0, 2).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().mode,
        GpioMode::Input(PullMode::PullDown)
    );

    // Gpio Read
    assert_eq!(
        gpio.command(GPIO_READ_INPUT, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        gpio.command(GPIO_READ_INPUT, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );

    assert_eq!(gpio.set_value(0, true), Ok(()));
    assert_eq!(
        gpio.command(GPIO_READ_INPUT, 0, 0).get_success_u32(),
        Some(1)
    );
    assert_eq!(gpio.set_value(0, false), Ok(()));
    assert_eq!(
        gpio.command(GPIO_READ_INPUT, 0, 0).get_success_u32(),
        Some(0)
    );

    // Enable Interrupts
    assert_eq!(
        gpio.command(GPIO_ENABLE_INTERRUPTS, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        gpio.command(GPIO_ENABLE_INTERRUPTS, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert_eq!(
        gpio.command(GPIO_ENABLE_INTERRUPTS, 0, 3).get_failure(),
        Some(ErrorCode::Invalid)
    );

    assert!(gpio.command(GPIO_ENABLE_INTERRUPTS, 0, 0).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().interrupt_enabled,
        Some(InterruptEdge::Either)
    );

    assert!(gpio.command(GPIO_ENABLE_INTERRUPTS, 0, 1).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().interrupt_enabled,
        Some(InterruptEdge::Rising)
    );

    assert!(gpio.command(GPIO_ENABLE_INTERRUPTS, 0, 2).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().interrupt_enabled,
        Some(InterruptEdge::Falling)
    );

    // Disable Interrupts
    assert_eq!(
        gpio.command(GPIO_DISABLE_INTERRUPTS, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        gpio.command(GPIO_DISABLE_INTERRUPTS, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );

    assert!(gpio.command(GPIO_DISABLE_INTERRUPTS, 0, 0).is_success());
    assert_eq!(gpio.get_gpio_state(0).unwrap().interrupt_enabled, None);

    // Disable Pin
    assert_eq!(
        gpio.command(GPIO_DISABLE, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        gpio.command(GPIO_DISABLE, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );

    assert!(gpio.command(GPIO_DISABLE, 0, 0).is_success());
    assert_eq!(gpio.get_gpio_state(0).unwrap().mode, GpioMode::Disable);
}

// Integration test that verifies Gpio works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let gpio = Gpio::<10>::new();
    gpio.set_missing_gpio(1);
    kernel.add_driver(&gpio);
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_COUNT, 1, 2).get_success_u32(),
        Some(10)
    );

    // Enable Output
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_OUTPUT, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_OUTPUT, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_OUTPUT, 0, 0).is_success());

    // Gpio Set
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_SET, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_SET, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_SET, 0, 0).is_success(),);
    assert!(gpio.get_gpio_state(0).unwrap().value);

    // Gpio Clear
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_CLEAR, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_CLEAR, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_CLEAR, 0, 0).is_success(),);
    assert!(!gpio.get_gpio_state(0).unwrap().value);

    // Gpio Toggle
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_TOGGLE, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_TOGGLE, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_TOGGLE, 0, 0).is_success(),);
    assert!(gpio.get_gpio_state(0).unwrap().value);
    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_TOGGLE, 0, 0).is_success(),);
    assert!(!gpio.get_gpio_state(0).unwrap().value);

    // Enable Input
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INPUT, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INPUT, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );

    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INPUT, 0, 3).get_failure(),
        Some(ErrorCode::Invalid)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INPUT, 0, 0).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().mode,
        GpioMode::Input(PullMode::PullNone)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INPUT, 0, 1).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().mode,
        GpioMode::Input(PullMode::PullUp)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INPUT, 0, 2).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().mode,
        GpioMode::Input(PullMode::PullDown)
    );

    // Gpio Read
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_READ_INPUT, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_READ_INPUT, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );

    assert_eq!(gpio.set_value(0, true), Ok(()));
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_READ_INPUT, 0, 0).get_success_u32(),
        Some(1)
    );
    assert_eq!(gpio.set_value(0, false), Ok(()));
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_READ_INPUT, 0, 0).get_success_u32(),
        Some(0)
    );

    // Enable Interrupts
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INTERRUPTS, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INTERRUPTS, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INTERRUPTS, 0, 3).get_failure(),
        Some(ErrorCode::Invalid)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INTERRUPTS, 0, 0).is_success());
    assert_eq!(
        gpio.get_gpio_state(0).unwrap().interrupt_enabled,
        Some(InterruptEdge::Either)
    );

    let listener = Cell::<Option<(u32, u32)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        assert_eq!(gpio.set_value(0, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        assert_eq!(gpio.set_value(0, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(gpio.set_value(0, false), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        assert_eq!(gpio.set_value(0, false), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INTERRUPTS, 0, 1).is_success());
        assert_eq!(
            gpio.get_gpio_state(0).unwrap().interrupt_enabled,
            Some(InterruptEdge::Rising)
        );

        assert_eq!(gpio.set_value(0, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        assert_eq!(gpio.set_value(0, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(gpio.set_value(0, false), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_ENABLE_INTERRUPTS, 0, 2).is_success());
        assert_eq!(
            gpio.get_gpio_state(0).unwrap().interrupt_enabled,
            Some(InterruptEdge::Falling)
        );

        assert_eq!(gpio.set_value(0, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(gpio.set_value(0, false), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        assert_eq!(gpio.set_value(0, false), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });

    // Disable Interrupts
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_DISABLE_INTERRUPTS, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_DISABLE_INTERRUPTS, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_DISABLE_INTERRUPTS, 0, 0).is_success());
    assert_eq!(gpio.get_gpio_state(0).unwrap().interrupt_enabled, None);

    let listener = Cell::<Option<(u32, u32)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        assert_eq!(gpio.set_value(0, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(gpio.set_value(0, false), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });

    // Disable Pin
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_DISABLE, 11, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GPIO_DISABLE, 1, 0).get_failure(),
        Some(ErrorCode::NoDevice)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, GPIO_DISABLE, 0, 0).is_success());
    assert_eq!(gpio.get_gpio_state(0).unwrap().mode, GpioMode::Disable);
}
