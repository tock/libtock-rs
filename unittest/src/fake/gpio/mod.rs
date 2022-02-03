//! Fake implementation of the GPIO API, documented here:
//! https://github.com/tock/tock/blob/master/doc/syscalls/00004_gpio.md
//!
//! Like the real API, `Gpio` controls a set of fake gpios. It provides
//! a function `get_button_state` used to retrieve the state and interrupt
//! status of a button.
//!
//! It also provides the function `set_pressed` that set the button's state.

use core::cell::Cell;
use libtock_platform::{CommandReturn, ErrorCode};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GpioMode {
    Output,
    Input(PullMode),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PullMode {
    PullNone = 0,
    PullUp = 1,
    PullDown = 2,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GpioState {
    pub value: bool,
    pub mode: GpioMode,
    pub interrupt_enabled: bool,
}

pub struct Gpio<const NUM_GPIOS: usize> {
    gpios: [Cell<GpioState>; NUM_GPIOS],
}

impl<const NUM_GPIOS: usize> Gpio<NUM_GPIOS> {
    pub fn new() -> std::rc::Rc<Gpio<NUM_GPIOS>> {
        #[allow(clippy::declare_interior_mutable_const)]
        const OFF: Cell<GpioState> = Cell::new(GpioState {
            value: false,
            mode: GpioMode::Input(PullMode::PullNone),
            interrupt_enabled: false,
        });
        std::rc::Rc::new(Gpio {
            gpios: [OFF; NUM_GPIOS],
        })
    }
}

impl<const NUM_GPIOS: usize> crate::fake::SyscallDriver for Gpio<NUM_GPIOS> {
    fn id(&self) -> u32 {
        DRIVER_ID
    }
    fn num_upcalls(&self) -> u32 {
        1
    }

    fn command(&self, command_number: u32, argument0: u32, _argument1: u32) -> CommandReturn {
        if command_number == GPIO_COUNT {
            crate::command_return::success_u32(NUM_GPIOS as u32)
        } else {
            if argument0 < NUM_GPIOS as u32 {
                match command_number {
                    GPIO_ENABLE_OUTPUT => {
                        let gpio = self.gpios[argument0 as usize].get();
                        self.gpios[argument0 as usize].set(GpioState {
                            mode: GpioMode::Output,
                            ..gpio
                        });
                        crate::command_return::success()
                    }
                    GPIO_SET => {
                        let gpio = self.gpios[argument0 as usize].get();
                        self.gpios[argument0 as usize].set(GpioState {
                            mode: GpioMode::Output,
                            ..gpio
                        });
                        crate::command_return::success()
                    }
                    _ => crate::command_return::failure(ErrorCode::NoSupport),
                }
            } else {
                crate::command_return::failure(ErrorCode::Invalid)
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Implementation details below
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests;

const DRIVER_ID: u32 = 4;

// Command IDs
const GPIO_COUNT: u32 = 0;

const GPIO_ENABLE_OUTPUT: u32 = 1;
const GPIO_SET: u32 = 2;
const GPIO_CLEAR: u32 = 3;
const GPIO_TOGGLE: u32 = 4;

const GPIO_ENABLE_INPUT: u32 = 5;
const GPIO_READ_INPUT: u32 = 6;
const GPIO_DISABLE: u32 = 9;

impl<const NUM_GPIOS: usize> Gpio<NUM_GPIOS> {
    pub fn set_pressed(&self, button: u32, pressed: bool) -> Result<(), ErrorCode> {
        self.buttons
            .get(button as usize)
            .map(|button| {
                button.set(GpioState {
                    pressed,
                    ..button.get()
                })
            })
            .ok_or(ErrorCode::Invalid)
    }

    pub fn get_button_state(&self, button: u32) -> Option<GpioState> {
        self.buttons.get(button as usize).map(|button| button.get())
    }
}
