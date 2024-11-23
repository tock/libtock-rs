//! Fake implementation of the LEDs API, documented here:
//! https://github.com/tock/tock/blob/master/doc/syscalls/00002_leds.md
//!
//! Like the real API, `Leds` controls a set of fake LEDs. It provides
//! a function `get_led` used to retrieve the state of an LED.

use crate::DriverInfo;
use core::cell::Cell;
use libtock_platform::{CommandReturn, ErrorCode};

pub struct Leds<const LEDS_COUNT: usize> {
    leds: [Cell<bool>; LEDS_COUNT],
}

impl<const LEDS_COUNT: usize> Leds<LEDS_COUNT> {
    pub fn new() -> std::rc::Rc<Leds<LEDS_COUNT>> {
        #[allow(clippy::declare_interior_mutable_const)]
        const OFF: Cell<bool> = Cell::new(false);
        std::rc::Rc::new(Leds {
            leds: [OFF; LEDS_COUNT],
        })
    }

    pub fn get_led(&self, led: u32) -> Option<bool> {
        self.leds.get(led as usize).map(|led| led.get())
    }
}

impl<const LEDS_COUNT: usize> crate::fake::SyscallDriver for Leds<LEDS_COUNT> {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM)
    }

    fn command(&self, command_num: u32, argument0: u32, _argument1: u32) -> CommandReturn {
        match command_num {
            EXISTS => crate::command_return::success_u32(LEDS_COUNT as u32),
            LED_ON => {
                if argument0 < LEDS_COUNT as u32 {
                    self.leds[argument0 as usize].set(true);
                    crate::command_return::success()
                } else {
                    crate::command_return::failure(ErrorCode::Invalid)
                }
            }
            LED_OFF => {
                if argument0 < LEDS_COUNT as u32 {
                    self.leds[argument0 as usize].set(false);
                    crate::command_return::success()
                } else {
                    crate::command_return::failure(ErrorCode::Invalid)
                }
            }
            LED_TOGGLE => {
                if argument0 < LEDS_COUNT as u32 {
                    self.leds[argument0 as usize].set(!self.leds[argument0 as usize].get());
                    crate::command_return::success()
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
// Implementation details below
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x2;

// Command numbers
const EXISTS: u32 = 0;
const LED_ON: u32 = 1;
const LED_OFF: u32 = 2;
const LED_TOGGLE: u32 = 3;
