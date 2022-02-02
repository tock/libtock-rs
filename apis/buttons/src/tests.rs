use core::cell::Cell;

use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::{fake, upcall};

use crate::{ButtonListener, DRIVER_ID};

use super::ButtonState;

type Buttons = super::Buttons<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Buttons::count(), Err(ErrorCode::NoDevice));
}

#[test]
fn num_buttons() {
    let kernel = fake::Kernel::new();
    let driver = fake::Buttons::<10>::new();
    kernel.add_driver(&driver);
    assert_eq!(Buttons::count(), Ok(10));
}

#[test]
fn read() {
    let kernel = fake::Kernel::new();
    let driver = fake::Buttons::<10>::new();
    kernel.add_driver(&driver);

    assert_eq!(driver.set_pressed(0, true), Ok(()));
    assert_eq!(Buttons::read(0), Ok(ButtonState::Pressed));

    assert_eq!(driver.set_pressed(0, false), Ok(()));
    assert_eq!(Buttons::read(0), Ok(ButtonState::Released));

    assert_eq!(Buttons::read(11), Err(ErrorCode::Invalid));
}

#[test]
fn interrupts() {
    let kernel = fake::Kernel::new();
    let driver = fake::Buttons::<10>::new();
    kernel.add_driver(&driver);

    assert_eq!(Buttons::enable_interrupts(0), Ok(()));
    assert_eq!(driver.get_button_state(0).unwrap().interrupt_enabled, true);

    assert_eq!(Buttons::disable_interrupts(0), Ok(()));
    assert_eq!(driver.get_button_state(0).unwrap().interrupt_enabled, false);

    assert_eq!(Buttons::enable_interrupts(11), Err(ErrorCode::Invalid));
    assert_eq!(Buttons::disable_interrupts(11), Err(ErrorCode::Invalid));
}

#[test]
fn subscribe() {
    let kernel = fake::Kernel::new();
    let driver = fake::Buttons::<10>::new();
    kernel.add_driver(&driver);

    let pressed_interrupt_fired: Cell<bool> = Cell::new(false);
    let listener = ButtonListener(|button, state| {
        assert_eq!(button, 0);
        assert_eq!(state, ButtonState::Pressed);
        pressed_interrupt_fired.set(true);
    });
    share::scope(|subscribe| {
        assert_eq!(Buttons::register_listener(&listener, subscribe), Ok(()));
        upcall::schedule(DRIVER_ID, 0, (0, 1, 0)).unwrap();
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
    assert_eq!(pressed_interrupt_fired.get(), true);

    let pressed_interrupt_fired: Cell<bool> = Cell::new(false);
    let listener = ButtonListener(|button, state| {
        assert_eq!(button, 0);
        assert_eq!(state, ButtonState::Released);
        pressed_interrupt_fired.set(true);
    });
    share::scope(|subscribe| {
        assert_eq!(Buttons::register_listener(&listener, subscribe), Ok(()));
        upcall::schedule(DRIVER_ID, 0, (0, 0, 0)).unwrap();
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
    assert_eq!(pressed_interrupt_fired.get(), true);

    let pressed_interrupt_fired: Cell<bool> = Cell::new(false);
    let listener = ButtonListener(|_, _| {
        pressed_interrupt_fired.set(true);
    });
    share::scope(|subscribe| {
        assert_eq!(Buttons::register_listener(&listener, subscribe), Ok(()));
        Buttons::unregister_listener();
        upcall::schedule(DRIVER_ID, 0, (0, 1, 0)).unwrap();
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
    assert_eq!(pressed_interrupt_fired.get(), false);
}
