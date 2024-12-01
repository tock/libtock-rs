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
use std::convert::TryFrom;

use crate::{DriverInfo, DriverShareRef};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GpioMode {
    Output,
    Input(PullMode),
    Disable,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PullMode {
    PullNone = 0,
    PullUp = 1,
    PullDown = 2,
}

impl TryFrom<u32> for PullMode {
    type Error = ErrorCode;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PullMode::PullNone),
            1 => Ok(PullMode::PullUp),
            2 => Ok(PullMode::PullDown),
            _ => Err(ErrorCode::Invalid),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InterruptEdge {
    Either,
    Rising,
    Falling,
}

impl TryFrom<u32> for InterruptEdge {
    type Error = ErrorCode;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InterruptEdge::Either),
            1 => Ok(InterruptEdge::Rising),
            2 => Ok(InterruptEdge::Falling),
            _ => Err(ErrorCode::Invalid),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GpioState {
    pub value: bool,
    pub mode: GpioMode,
    pub interrupt_enabled: Option<InterruptEdge>,
}

pub struct Gpio<const NUM_GPIOS: usize> {
    gpios: [Cell<Option<GpioState>>; NUM_GPIOS],
    share_ref: DriverShareRef,
}

impl<const NUM_GPIOS: usize> Gpio<NUM_GPIOS> {
    pub fn new() -> std::rc::Rc<Gpio<NUM_GPIOS>> {
        #[allow(clippy::declare_interior_mutable_const)]
        const OFF: Cell<Option<GpioState>> = Cell::new(Some(GpioState {
            value: false,
            mode: GpioMode::Input(PullMode::PullNone),
            interrupt_enabled: None,
        }));
        std::rc::Rc::new(Gpio {
            gpios: [OFF; NUM_GPIOS],
            share_ref: Default::default(),
        })
    }

    pub fn set_missing_gpio(&self, gpio: usize) {
        if let Some(state) = self.gpios.get(gpio) {
            state.set(None);
        }
    }

    pub fn set_value(&self, pin: u32, value: bool) -> Result<(), ErrorCode> {
        self.gpios
            .get(pin as usize)
            .map(|gpio| {
                if let Some(gpio_state) = gpio.get() {
                    let original_value = gpio_state.value;
                    gpio.set(Some(GpioState {
                        value,
                        ..gpio_state
                    }));
                    if original_value != value {
                        if value {
                            if gpio_state.interrupt_enabled == Some(InterruptEdge::Either)
                                || gpio_state.interrupt_enabled == Some(InterruptEdge::Rising)
                            {
                                self.share_ref
                                    .schedule_upcall(0, (pin, value as u32, 0))
                                    .expect("Unable to schedule upcall");
                            }
                        } else if gpio_state.interrupt_enabled == Some(InterruptEdge::Falling)
                            || gpio_state.interrupt_enabled == Some(InterruptEdge::Either)
                        {
                            self.share_ref
                                .schedule_upcall(0, (pin, value as u32, 0))
                                .expect("Unable to schedule upcall");
                        }
                    }
                    Ok(())
                } else {
                    Err(ErrorCode::NoDevice)
                }
            })
            .ok_or(ErrorCode::Invalid)
            .and_then(|value| value)
    }

    pub fn get_gpio_state(&self, button: u32) -> Option<GpioState> {
        self.gpios
            .get(button as usize)
            .map(|button| button.get())
            .and_then(|value| value)
    }
}

impl<const NUM_GPIOS: usize> crate::fake::SyscallDriver for Gpio<NUM_GPIOS> {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_number: u32, argument0: u32, argument1: u32) -> CommandReturn {
        if command_number == EXISTS {
            crate::command_return::success()
        } else if command_number == GPIO_COUNT {
            crate::command_return::success_u32(NUM_GPIOS as u32)
        } else if argument0 < NUM_GPIOS as u32 {
            if self.gpios[argument0 as usize].get().is_some() {
                let gpio = self.gpios[argument0 as usize].get().unwrap();
                match command_number {
                    GPIO_ENABLE_OUTPUT => {
                        self.gpios[argument0 as usize].set(Some(GpioState {
                            mode: GpioMode::Output,
                            ..gpio
                        }));
                        crate::command_return::success()
                    }
                    GPIO_SET => {
                        if let GpioMode::Output = gpio.mode {
                            self.gpios[argument0 as usize].set(Some(GpioState {
                                value: true,
                                ..gpio
                            }));
                        }
                        crate::command_return::success()
                    }
                    GPIO_CLEAR => {
                        if let GpioMode::Output = gpio.mode {
                            self.gpios[argument0 as usize].set(Some(GpioState {
                                value: false,
                                ..gpio
                            }));
                        }
                        crate::command_return::success()
                    }
                    GPIO_TOGGLE => {
                        if let GpioMode::Output = gpio.mode {
                            self.gpios[argument0 as usize].set(Some(GpioState {
                                value: !gpio.value,
                                ..gpio
                            }));
                        }
                        crate::command_return::success()
                    }
                    GPIO_ENABLE_INPUT => {
                        let pull_mode = PullMode::try_from(argument1);
                        match pull_mode {
                            Ok(mode) => {
                                self.gpios[argument0 as usize].set(Some(GpioState {
                                    mode: GpioMode::Input(mode),
                                    ..gpio
                                }));
                                crate::command_return::success()
                            }
                            Err(error) => crate::command_return::failure(error),
                        }
                    }
                    GPIO_READ_INPUT => {
                        if let GpioMode::Input(_) = gpio.mode {
                            crate::command_return::success_u32(gpio.value as u32)
                        } else {
                            crate::command_return::success_u32(0)
                        }
                    }
                    GPIO_ENABLE_INTERRUPTS => {
                        let edge = InterruptEdge::try_from(argument1);
                        match edge {
                            Ok(interrupt_edge) => {
                                self.gpios[argument0 as usize].set(Some(GpioState {
                                    interrupt_enabled: Some(interrupt_edge),
                                    ..gpio
                                }));
                                crate::command_return::success()
                            }
                            Err(error) => crate::command_return::failure(error),
                        }
                    }
                    GPIO_DISABLE_INTERRUPTS => {
                        self.gpios[argument0 as usize].set(Some(GpioState {
                            interrupt_enabled: None,
                            ..gpio
                        }));
                        crate::command_return::success()
                    }
                    GPIO_DISABLE => {
                        self.gpios[argument0 as usize].set(Some(GpioState {
                            mode: GpioMode::Disable,
                            ..gpio
                        }));
                        crate::command_return::success()
                    }
                    _ => crate::command_return::failure(ErrorCode::NoSupport),
                }
            } else {
                crate::command_return::failure(ErrorCode::NoDevice)
            }
        } else {
            crate::command_return::failure(ErrorCode::Invalid)
        }
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x4;

// Command IDs
const EXISTS: u32 = 0;

const GPIO_ENABLE_OUTPUT: u32 = 1;
const GPIO_SET: u32 = 2;
const GPIO_CLEAR: u32 = 3;
const GPIO_TOGGLE: u32 = 4;

const GPIO_ENABLE_INPUT: u32 = 5;
const GPIO_READ_INPUT: u32 = 6;

const GPIO_ENABLE_INTERRUPTS: u32 = 7;
const GPIO_DISABLE_INTERRUPTS: u32 = 8;

const GPIO_DISABLE: u32 = 9;

const GPIO_COUNT: u32 = 10;
