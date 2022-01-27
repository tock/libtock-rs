#![no_std]

use core::{convert::TryFrom, marker::PhantomData};

use libtock_platform::{ErrorCode, Syscalls};

/// The LEDs driver
///
/// # Example
/// ```ignore
/// use libtock2::Leds;
///
/// // Turn on led 0
/// Leds::on(0);
/// ```
pub struct Gpio<S: Syscalls>(S);

impl<S: Syscalls> Gpio<S> {
    /// Run a check against the gpio capsule to ensure it is present.
    ///
    /// Returns true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    pub fn driver_check() -> bool {
        S::command(DRIVER_ID, GPIO_COUNT, 0, 0).is_success_u32()
    }

    pub fn gpios() -> Gpios<'static, S> {
        let num_gpios = S::command(DRIVER_ID, GPIO_COUNT, 0, 0)
            .get_success_u32()
            .unwrap_or_default() as usize;
        Gpios {
            num_gpios,
            current_gpio: 0,
            _syscalls: &PhantomData,
        }
    }
}

pub enum GpioState {
    Low = 0,
    High = 1,
}

impl From<u32> for GpioState {
    fn from(original: u32) -> GpioState {
        match original {
            0 => GpioState::Low,
            _ => GpioState::High,
        }
    }
}

pub enum Error {
    Invalid,
    Failed,
}

pub trait Pull {
    const MODE: u32;
}

pub struct PullUp;
impl Pull for PullUp {
    const MODE: u32 = 1;
}

pub struct PullDown;
impl Pull for PullDown {
    const MODE: u32 = 2;
}

pub struct PullNone;
impl Pull for PullNone {
    const MODE: u32 = 0;
}

pub struct Pin<'a, S: Syscalls> {
    pin_number: u32,
    _gpio: &'a PhantomData<S>,
}

impl<'a, S: Syscalls> Pin<'a, S> {
    pub(crate) fn make_output(&'a self) -> Result<OutputPin<'a, S>, ErrorCode> {
        if let Some(error) =
            S::command(DRIVER_ID, GPIO_ENABLE_OUTPUT, self.pin_number, 0).get_failure()
        {
            return Err(error);
        }
        Ok(OutputPin { pin: self })
    }

    pub(crate) fn make_input<P: Pull>(&'a self) -> Result<InputPin<'a, S, P>, ErrorCode> {
        if let Some(error) =
            S::command(DRIVER_ID, GPIO_ENABLE_INPUT, self.pin_number, P::MODE).get_failure()
        {
            return Err(error);
        }
        Ok(InputPin {
            pin: self,
            _pull: PhantomData,
        })
    }
}

pub struct OutputPin<'a, S: Syscalls> {
    pin: &'a Pin<'a, S>,
}

impl<'a, S: Syscalls> OutputPin<'a, S> {
    pub fn toggle(&mut self) -> Result<(), ErrorCode> {
        if let Some(error) =
            S::command(DRIVER_ID, GPIO_TOGGLE, self.pin.pin_number, 0).get_failure()
        {
            Err(error)
        } else {
            Ok(())
        }
    }
    pub fn set(&mut self) -> Result<(), ErrorCode> {
        if let Some(error) = S::command(DRIVER_ID, GPIO_SET, self.pin.pin_number, 0).get_failure() {
            Err(error)
        } else {
            Ok(())
        }
    }
    pub fn clear(&mut self) -> Result<(), ErrorCode> {
        if let Some(error) = S::command(DRIVER_ID, GPIO_CLEAR, self.pin.pin_number, 0).get_failure()
        {
            Err(error)
        } else {
            Ok(())
        }
    }
}

pub struct InputPin<'a, S: Syscalls, P: Pull> {
    pin: &'a Pin<'a, S>,
    _pull: PhantomData<P>,
}

impl<'a, S: Syscalls, P: Pull> InputPin<'a, S, P> {
    pub fn read(&self) -> Option<GpioState> {
        S::command(DRIVER_ID, GPIO_READ_INPUT, self.pin.pin_number, 0)
            .get_success_u32()
            .map(|v| v.into())
    }
}

impl<'a, S: Syscalls> TryFrom<&'a Pin<'a, S>> for OutputPin<'a, S> {
    type Error = ErrorCode;

    fn try_from(pin: &'a Pin<'a, S>) -> Result<OutputPin<'a, S>, ErrorCode> {
        pin.make_output()
    }
}

impl<'a, S: Syscalls, P: Pull> TryFrom<&'a Pin<'a, S>> for InputPin<'a, S, P> {
    type Error = ErrorCode;

    fn try_from(pin: &'a Pin<'a, S>) -> Result<InputPin<'a, S, P>, ErrorCode> {
        pin.make_input()
    }
}

pub struct Gpios<'a, S: Syscalls> {
    num_gpios: usize,
    current_gpio: usize,
    _syscalls: &'a PhantomData<S>,
}

impl<'a, S: Syscalls> Iterator for Gpios<'a, S> {
    type Item = Pin<'a, S>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_gpio < self.num_gpios {
            let pin_number = self.current_gpio as u32;
            self.current_gpio += 1;
            Some(Pin {
                pin_number,
                _gpio: &PhantomData,
            })
        } else {
            None
        }
    }
}

impl<'a, S: Syscalls> Drop for Pin<'a, S> {
    fn drop(&mut self) {
        S::command(DRIVER_ID, GPIO_DISABLE, self.pin_number, 0);
    }
}

const DRIVER_ID: u32 = 2;

// Command IDs
const GPIO_COUNT: u32 = 0;

const GPIO_ENABLE_OUTPUT: u32 = 1;
const GPIO_SET: u32 = 2;
const GPIO_CLEAR: u32 = 3;
const GPIO_TOGGLE: u32 = 4;

const GPIO_ENABLE_INPUT: u32 = 5;
const GPIO_READ_INPUT: u32 = 6;
const GPIO_DISABLE: u32 = 9;

#[cfg(test)]
mod tests;
