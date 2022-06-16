use core::cell::Cell;

use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

use crate::{ButtonListener, ButtonState};

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
    assert!(driver.get_button_state(0).unwrap().interrupt_enabled);

    assert_eq!(Buttons::disable_interrupts(0), Ok(()));
    assert!(!driver.get_button_state(0).unwrap().interrupt_enabled);

    assert_eq!(Buttons::enable_interrupts(11), Err(ErrorCode::Invalid));
    assert_eq!(Buttons::disable_interrupts(11), Err(ErrorCode::Invalid));
}

#[test]
fn subscribe() {
    let kernel = fake::Kernel::new();
    let driver = fake::Buttons::<10>::new();
    kernel.add_driver(&driver);

    let pressed_interrupt_count: Cell<bool> = Cell::new(false);
    let listener = ButtonListener(|button, state| {
        assert_eq!(button, 0);
        assert_eq!(state, ButtonState::Pressed);
        pressed_interrupt_count.set(true);
    });
    assert_eq!(Buttons::enable_interrupts(0), Ok(()));
    share::scope(|subscribe| {
        assert_eq!(Buttons::register_listener(&listener, subscribe), Ok(()));
        assert_eq!(driver.set_pressed(0, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
    assert!(pressed_interrupt_count.get());

    let pressed_interrupt_count: Cell<u32> = Cell::new(0);
    let expected_button_state: Cell<ButtonState> = Cell::new(ButtonState::Released);
    let listener = ButtonListener(|button, state| {
        assert_eq!(button, 1);
        assert_eq!(state, expected_button_state.get());
        pressed_interrupt_count.set(pressed_interrupt_count.get() + 1);
    });
    share::scope(|subscribe| {
        assert_eq!(Buttons::enable_interrupts(1), Ok(()));
        assert_eq!(Buttons::register_listener(&listener, subscribe), Ok(()));
        expected_button_state.set(ButtonState::Pressed);
        assert_eq!(driver.set_pressed(1, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(driver.set_pressed(1, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        expected_button_state.set(ButtonState::Released);
        assert_eq!(driver.set_pressed(1, false), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(driver.set_pressed(1, false), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(Buttons::disable_interrupts(1), Ok(()));
        assert_eq!(driver.set_pressed(1, true), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
        assert_eq!(driver.set_pressed(1, false), Ok(()));
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
    assert_eq!(pressed_interrupt_count.get(), 2);

    let pressed_interrupt_count: Cell<bool> = Cell::new(false);
    let listener = ButtonListener(|_, _| {
        pressed_interrupt_count.set(true);
    });
    share::scope(|subscribe| {
        assert_eq!(Buttons::enable_interrupts(0), Ok(()));
        assert_eq!(Buttons::register_listener(&listener, subscribe), Ok(()));
        Buttons::unregister_listener();
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
    assert!(!pressed_interrupt_count.get());
}
