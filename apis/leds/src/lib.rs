#![no_std]

use libtock_platform::{ErrorCode, Syscalls};

/// The LEDs driver
///
/// # Example
/// ```ignore
/// use libtock2::LedsFactory;
///
/// let mut leds_factory = LedsFactory::new();
/// let leds_driver = leds_factory.init_driver()?;
/// // Turn on led 0
/// leds_driver.get(0)?.on()
/// ```
use core::marker::PhantomData;

mod command_nr {
    pub const COUNT: u32 = 0;
    pub const ON: u32 = 1;
    pub const OFF: u32 = 2;
    pub const TOGGLE: u32 = 3;
}

#[non_exhaustive]
pub struct LedsFactory<S: Syscalls>(PhantomData<S>);

impl<S: Syscalls> LedsFactory<S> {
    pub fn init_driver(&mut self) -> Result<LedsDriver<S>, ErrorCode> {
        let num_leds = S::command(DRIVER_ID, command_nr::COUNT, 0, 0);
        if num_leds.is_success_u32() {
            let driver = LedsDriver {
                num_leds: num_leds.get_success_u32().unwrap_or_default() as usize,
                lifetime: PhantomData,
            };
            Ok(driver)
        } else {
            Err(num_leds.get_failure().unwrap_or(ErrorCode::Fail))
        }
    }

    // temporary placeholder here so that driver is usable
    // until a driver imfrastrucrure is put in place
    #[allow(clippy::new_without_default)]
    pub fn new() -> LedsFactory<S> {
        LedsFactory(PhantomData)
    }
}

pub struct LedsDriver<'a, S: Syscalls> {
    num_leds: usize,
    lifetime: PhantomData<&'a S>,
}

impl<'a, S: Syscalls> LedsDriver<'a, S> {
    pub fn num_leds(&self) -> usize {
        self.num_leds
    }

    pub fn leds(&self) -> Leds<S> {
        Leds {
            num_leds: self.num_leds,
            curr_led: 0,
            lifetime: PhantomData,
        }
    }

    /// Returns the led at 0-based index `led_num`
    pub fn get(&self, led_num: usize) -> Result<Led<S>, ErrorCode> {
        if led_num < self.num_leds {
            Ok(Led {
                led_num,
                lifetime: PhantomData,
            })
        } else {
            Err(ErrorCode::Invalid)
        }
    }
}

pub struct Leds<'a, S: Syscalls> {
    num_leds: usize,
    curr_led: usize,
    lifetime: PhantomData<&'a S>,
}

impl<'a, S: Syscalls> Iterator for Leds<'a, S> {
    type Item = Led<'a, S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_led < self.num_leds {
            let item = Led {
                led_num: self.curr_led,
                lifetime: PhantomData,
            };
            self.curr_led += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct Led<'a, S: Syscalls> {
    led_num: usize,
    lifetime: PhantomData<&'a S>,
}

impl<'a, S: Syscalls> Led<'a, S> {
    pub fn led_num(&self) -> usize {
        self.led_num
    }

    pub fn set(&self, state: impl Into<LedState>) -> Result<(), ErrorCode> {
        match state.into() {
            LedState::On => self.on(),
            LedState::Off => self.off(),
        }
    }

    pub fn on(&self) -> Result<(), ErrorCode> {
        let value = S::command(DRIVER_ID, command_nr::ON, self.led_num as u32, 0);
        if value.is_failure() {
            Err(value.get_failure().unwrap_or(ErrorCode::Fail))
        } else {
            Ok(())
        }
    }

    pub fn off(&self) -> Result<(), ErrorCode> {
        let value = S::command(DRIVER_ID, command_nr::OFF, self.led_num as u32, 0);
        if value.is_failure() {
            Err(value.get_failure().unwrap_or(ErrorCode::Fail))
        } else {
            Ok(())
        }
    }

    pub fn toggle(&self) -> Result<(), ErrorCode> {
        let value = S::command(DRIVER_ID, command_nr::TOGGLE, self.led_num as u32, 0);
        if value.is_failure() {
            Err(value.get_failure().unwrap_or(ErrorCode::Fail))
        } else {
            Ok(())
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LedState {
    On,
    Off,
}

impl From<bool> for LedState {
    fn from(from_value: bool) -> Self {
        if from_value {
            LedState::On
        } else {
            LedState::Off
        }
    }
}

const DRIVER_ID: u32 = 2;

#[cfg(test)]
mod tests;
