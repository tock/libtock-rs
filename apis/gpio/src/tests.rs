use core::cell::Cell;

use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::{
    fake::{self, GpioMode, InterruptEdge, PullMode},
    upcall,
};

use crate::{GpioInterruptListener, GpioState, PullDown, PullNone, PullUp, DRIVER_NUM};

type Gpio = super::Gpio<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Gpio::count(), Err(ErrorCode::NoDevice));
}

#[test]
fn num_gpio() {
    let kernel = fake::Kernel::new();
    let driver = fake::Gpio::<10>::new();
    kernel.add_driver(&driver);
    assert_eq!(Gpio::count(), Ok(10));
}

// Tests the OutputPin implementation.
#[test]
fn output() {
    let kernel = fake::Kernel::new();
    let driver = fake::Gpio::<10>::new();
    driver.set_missing_gpio(1);
    kernel.add_driver(&driver);

    assert_eq!(Gpio::count(), Ok(10));

    let pin_11 = Gpio::get_pin(11);
    assert!(pin_11.is_err());
    let _ = pin_11.map_err(|error| assert_eq!(error, ErrorCode::Invalid));
    let pin_1 = Gpio::get_pin(1);
    assert!(pin_1.is_err());
    let _ = pin_1.map_err(|error| assert_eq!(error, ErrorCode::NoDevice));

    let pin_0 = Gpio::get_pin(0);
    assert!(pin_0.is_ok());

    let _ = pin_0.map(|mut pin| {
        let output_pin = pin.make_output();
        assert!(output_pin.is_ok());
        assert_eq!(driver.get_gpio_state(0).unwrap().mode, GpioMode::Output);
        let _ = output_pin.map(|mut pin| {
            assert_eq!(pin.set(), Ok(()));
            assert!(driver.get_gpio_state(0).unwrap().value);
            assert_eq!(pin.clear(), Ok(()));
            assert!(!driver.get_gpio_state(0).unwrap().value);
            assert_eq!(pin.toggle(), Ok(()));
            assert!(driver.get_gpio_state(0).unwrap().value);
            assert_eq!(pin.toggle(), Ok(()));
            assert!(!driver.get_gpio_state(0).unwrap().value);
        });
        assert_eq!(driver.get_gpio_state(0).unwrap().mode, GpioMode::Disable);
    });
}

// Tests the InputPin implementation
#[test]
fn input() {
    let kernel = fake::Kernel::new();
    let driver = fake::Gpio::<10>::new();
    driver.set_missing_gpio(1);
    kernel.add_driver(&driver);

    assert_eq!(Gpio::count(), Ok(10));

    let pin_11 = Gpio::get_pin(11);
    assert!(pin_11.is_err());
    let _ = pin_11.map_err(|error| assert_eq!(error, ErrorCode::Invalid));
    let pin_1 = Gpio::get_pin(1);
    assert!(pin_1.is_err());
    let _ = pin_1.map_err(|error| assert_eq!(error, ErrorCode::NoDevice));

    let pin_0 = Gpio::get_pin(0);
    assert!(pin_0.is_ok());

    let _ = pin_0.map(|pin| {
        let input_pin = pin.make_input::<PullNone>();
        assert!(input_pin.is_ok());
        assert_eq!(
            driver.get_gpio_state(0).unwrap().mode,
            GpioMode::Input(PullMode::PullNone)
        );

        let input_pin = pin.make_input::<PullUp>();
        assert!(input_pin.is_ok());
        assert_eq!(
            driver.get_gpio_state(0).unwrap().mode,
            GpioMode::Input(PullMode::PullUp)
        );

        let input_pin = pin.make_input::<PullDown>();
        assert!(input_pin.is_ok());
        assert_eq!(
            driver.get_gpio_state(0).unwrap().mode,
            GpioMode::Input(PullMode::PullDown)
        );

        let _ = input_pin.map(|pin| {
            assert_eq!(driver.set_value(0, true), Ok(()));
            assert_eq!(pin.read(), Ok(GpioState::High));
            assert_eq!(driver.set_value(0, false), Ok(()));
            assert_eq!(pin.read(), Ok(GpioState::Low));
        });
        assert_eq!(driver.get_gpio_state(0).unwrap().mode, GpioMode::Disable);
    });
}

// Tests the pin interrupts implementation
#[test]
fn interrupts() {
    let kernel = fake::Kernel::new();
    let driver = fake::Gpio::<10>::new();
    driver.set_missing_gpio(1);
    kernel.add_driver(&driver);

    assert_eq!(Gpio::count(), Ok(10));

    let gpio_state = Cell::<Option<GpioState>>::new(None);
    let listener = GpioInterruptListener(|gpio, state| {
        assert_eq!(gpio, 0);
        gpio_state.set(Some(state));
    });

    share::scope(|subscribe| {
        assert_eq!(Gpio::register_listener(&listener, subscribe), Ok(()));
        assert_eq!(upcall::schedule(DRIVER_NUM, 0, (0, 0, 0)), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(gpio_state.get(), Some(GpioState::Low));
    });

    assert_eq!(upcall::schedule(DRIVER_NUM, 0, (0, 0, 0)), Ok(()));
    assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

    let pin_11 = Gpio::get_pin(11);
    assert!(pin_11.is_err());
    let _ = pin_11.map_err(|error| assert_eq!(error, ErrorCode::Invalid));
    let pin_1 = Gpio::get_pin(1);
    assert!(pin_1.is_err());
    let _ = pin_1.map_err(|error| assert_eq!(error, ErrorCode::NoDevice));

    let pin_0 = Gpio::get_pin(0);
    assert!(pin_0.is_ok());

    let _ = pin_0.map(|pin| {
        // Either
        let input_pin = pin.make_input::<PullNone>();
        assert!(input_pin.is_ok());
        assert_eq!(
            driver.get_gpio_state(0).unwrap().mode,
            GpioMode::Input(PullMode::PullNone)
        );

        let _ = input_pin.map(|pin| {
            assert_eq!(
                pin.enable_interrupts(crate::PinInterruptEdge::Either),
                Ok(())
            );
            assert_eq!(
                driver.get_gpio_state(0).unwrap().interrupt_enabled,
                Some(InterruptEdge::Either)
            );

            assert_eq!(driver.set_value(0, false), Ok(()));

            let gpio_state = Cell::<Option<GpioState>>::new(None);
            let listener = GpioInterruptListener(|gpio, state| {
                assert_eq!(gpio, 0);
                gpio_state.set(Some(state));
            });

            share::scope(|subscribe| {
                assert_eq!(Gpio::register_listener(&listener, subscribe), Ok(()));
                assert_eq!(driver.set_value(0, true), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
                assert_eq!(gpio_state.get(), Some(GpioState::High));
                gpio_state.set(None);
                assert_eq!(driver.set_value(0, false), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
                assert_eq!(gpio_state.get(), Some(GpioState::Low));

                assert_eq!(pin.disable_interrupts(), Ok(()));
                assert_eq!(driver.set_value(0, true), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
                assert_eq!(driver.set_value(0, false), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
            });
        });

        // Rising
        let input_pin = pin.make_input::<PullNone>();
        assert!(input_pin.is_ok());
        assert_eq!(
            driver.get_gpio_state(0).unwrap().mode,
            GpioMode::Input(PullMode::PullNone)
        );

        let _ = input_pin.map(|pin| {
            assert_eq!(
                pin.enable_interrupts(crate::PinInterruptEdge::Rising),
                Ok(())
            );
            assert_eq!(
                driver.get_gpio_state(0).unwrap().interrupt_enabled,
                Some(InterruptEdge::Rising)
            );

            assert_eq!(driver.set_value(0, false), Ok(()));

            let gpio_state = Cell::<Option<GpioState>>::new(None);
            let listener = GpioInterruptListener(|gpio, state| {
                assert_eq!(gpio, 0);
                gpio_state.set(Some(state));
            });

            share::scope(|subscribe| {
                assert_eq!(Gpio::register_listener(&listener, subscribe), Ok(()));
                assert_eq!(driver.set_value(0, true), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
                assert_eq!(gpio_state.get(), Some(GpioState::High));
                assert_eq!(driver.set_value(0, false), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

                assert_eq!(pin.disable_interrupts(), Ok(()));
                assert_eq!(driver.set_value(0, true), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
                assert_eq!(driver.set_value(0, false), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
            });
        });

        // Falling
        let input_pin = pin.make_input::<PullNone>();
        assert!(input_pin.is_ok());
        assert_eq!(
            driver.get_gpio_state(0).unwrap().mode,
            GpioMode::Input(PullMode::PullNone)
        );

        let _ = input_pin.map(|pin| {
            assert_eq!(
                pin.enable_interrupts(crate::PinInterruptEdge::Falling),
                Ok(())
            );
            assert_eq!(
                driver.get_gpio_state(0).unwrap().interrupt_enabled,
                Some(InterruptEdge::Falling)
            );

            assert_eq!(driver.set_value(0, false), Ok(()));

            let gpio_state = Cell::<Option<GpioState>>::new(None);
            let listener = GpioInterruptListener(|gpio, state| {
                assert_eq!(gpio, 0);
                gpio_state.set(Some(state));
            });

            share::scope(|subscribe| {
                assert_eq!(Gpio::register_listener(&listener, subscribe), Ok(()));
                assert_eq!(driver.set_value(0, true), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
                assert_eq!(driver.set_value(0, false), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
                assert_eq!(gpio_state.get(), Some(GpioState::Low));

                assert_eq!(pin.disable_interrupts(), Ok(()));
                assert_eq!(driver.set_value(0, true), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
                assert_eq!(driver.set_value(0, false), Ok(()));
                assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
            });
        });
    });
}

// Tests the pin event subcribe implementation
#[test]
fn subscribe() {
    let kernel = fake::Kernel::new();
    let driver = fake::Gpio::<10>::new();
    driver.set_missing_gpio(1);
    kernel.add_driver(&driver);

    assert_eq!(Gpio::count(), Ok(10));

    let gpio_state = Cell::<Option<GpioState>>::new(None);
    let listener = GpioInterruptListener(|gpio, state| {
        assert_eq!(gpio, 0);
        gpio_state.set(Some(state));
    });

    share::scope(|subscribe| {
        assert_eq!(Gpio::register_listener(&listener, subscribe), Ok(()));
        assert_eq!(upcall::schedule(DRIVER_NUM, 0, (0, 0, 0)), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(gpio_state.get(), Some(GpioState::Low));
    });

    assert_eq!(upcall::schedule(DRIVER_NUM, 0, (0, 0, 0)), Ok(()));
    assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
}
