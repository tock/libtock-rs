//! Fake implementation of the Buttons API, documented here:
//! https://github.com/tock/tock/blob/master/doc/syscalls/00003_buttons.md
//!
//! Like the real API, `Buttons` controls a set of fake buttons. It provides
//! a function `get_button_state` used to retrieve the state and interrupt
//! status of a button.
//!
//! It also provides the function `set_pressed` that set the button's state.

use core::cell::Cell;
use libtock_platform::{CommandReturn, ErrorCode};

use crate::{DriverInfo, DriverShareRef};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ButtonState {
    pub pressed: bool,
    pub interrupt_enabled: bool,
}

pub struct Buttons<const NUM_BUTTONS: usize> {
    buttons: [Cell<ButtonState>; NUM_BUTTONS],
    share_ref: DriverShareRef,
}

impl<const NUM_BUTTONS: usize> Buttons<NUM_BUTTONS> {
    pub fn new() -> std::rc::Rc<Buttons<NUM_BUTTONS>> {
        #[allow(clippy::declare_interior_mutable_const)]
        const OFF: Cell<ButtonState> = Cell::new(ButtonState {
            pressed: false,
            interrupt_enabled: false,
        });
        std::rc::Rc::new(Buttons {
            buttons: [OFF; NUM_BUTTONS],
            share_ref: Default::default(),
        })
    }

    pub fn set_pressed(&self, button: u32, pressed: bool) -> Result<(), ErrorCode> {
        self.buttons
            .get(button as usize)
            .map(|button_state| {
                let original_button_state = button_state.get();
                button_state.set(ButtonState {
                    pressed,
                    ..original_button_state
                });
                if original_button_state.interrupt_enabled
                    && original_button_state.pressed != pressed
                {
                    self.share_ref
                        .schedule_upcall(0, (button, pressed as u32, 0))
                        .expect("Unable to schedule upcall {}");
                }
            })
            .ok_or(ErrorCode::Invalid)
    }

    pub fn get_button_state(&self, button: u32) -> Option<ButtonState> {
        self.buttons.get(button as usize).map(|button| button.get())
    }
}

impl<const NUM_BUTTONS: usize> crate::fake::SyscallDriver for Buttons<NUM_BUTTONS> {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_number: u32, argument0: u32, _argument1: u32) -> CommandReturn {
        match command_number {
            BUTTONS_COUNT => crate::command_return::success_u32(NUM_BUTTONS as u32),
            BUTTONS_ENABLE_INTERRUPTS => {
                if argument0 < NUM_BUTTONS as u32 {
                    let button = self.buttons[argument0 as usize].get();
                    self.buttons[argument0 as usize].set(ButtonState {
                        interrupt_enabled: true,
                        ..button
                    });
                    crate::command_return::success()
                } else {
                    crate::command_return::failure(ErrorCode::Invalid)
                }
            }
            BUTTONS_DISABLE_INTERRUPTS => {
                if argument0 < NUM_BUTTONS as u32 {
                    let button = self.buttons[argument0 as usize].get();
                    self.buttons[argument0 as usize].set(ButtonState {
                        interrupt_enabled: false,
                        ..button
                    });
                    crate::command_return::success()
                } else {
                    crate::command_return::failure(ErrorCode::Invalid)
                }
            }
            BUTTONS_READ => {
                if argument0 < NUM_BUTTONS as u32 {
                    crate::command_return::success_u32(
                        self.buttons[argument0 as usize].get().pressed as u32,
                    )
                } else {
                    crate::command_return::failure(ErrorCode::Invalid)
                }
            }
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x3;

// Command IDs
const BUTTONS_COUNT: u32 = 0;

const BUTTONS_ENABLE_INTERRUPTS: u32 = 1;
const BUTTONS_DISABLE_INTERRUPTS: u32 = 2;

const BUTTONS_READ: u32 = 3;
