#![no_std]

use core::marker::PhantomData;

use libtock_platform::{ErrorCode, Syscalls};

/// The Gpios driver
///
/// # Example
/// ```ignore
/// use libtock2::Gpios;
///
/// // Turn on led 0
/// let pin = Gpios::get_pin(0)?;
///
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

    fn enable_gpio_output(pin: u32) -> Result<(), ErrorCode> {
        if let Some(error) = S::command(DRIVER_ID, GPIO_ENABLE_OUTPUT, pin, 0).get_failure() {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn enable_gpio_input(pin: u32, mode: u32) -> Result<(), ErrorCode> {
        if let Some(error) = S::command(DRIVER_ID, GPIO_ENABLE_INPUT, pin, mode).get_failure() {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn write(pin: u32, state: GpioState) -> Result<(), ErrorCode> {
        let action = match state {
            GpioState::Low => GPIO_CLEAR,
            _ => GPIO_SET,
        };
        if let Some(error) = S::command(DRIVER_ID, action, pin, 0).get_failure() {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn read(pin: u32) -> Result<GpioState, ErrorCode> {
        let command_return = S::command(DRIVER_ID, GPIO_READ_INPUT, pin, 0);
        if let Some(error) = command_return.get_failure() {
            Err(error)
        } else {
            command_return
                .get_success_u32()
                .map(|v| Ok(v.into()))
                .unwrap_or(Err(ErrorCode::Fail)) // TODO replace with BadRValue
        }
    }

    fn toggle(pin: u32) -> Result<(), ErrorCode> {
        if let Some(error) = S::command(DRIVER_ID, GPIO_TOGGLE, pin, 0).get_failure() {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn disable(pin: u32) -> Result<(), ErrorCode> {
        if let Some(error) = S::command(DRIVER_ID, GPIO_DISABLE, pin, 0).get_failure() {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn get_pin(pin: u32) -> Result<Pin<S>, ErrorCode> {
        Self::disable(pin)?;
        Ok(Pin {
            pin_number: pin,
            _syscalls: PhantomData,
        })
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

pub struct Pin<S: Syscalls> {
    pin_number: u32,
    _syscalls: PhantomData<S>,
}

impl<S: Syscalls> Pin<S> {
    pub fn make_output(&mut self) -> Result<OutputPin<S>, ErrorCode> {
        Gpio::<S>::enable_gpio_output(self.pin_number)?;
        Ok(OutputPin { pin: self })
    }

    pub fn make_input<P: Pull>(&self) -> Result<InputPin<S, P>, ErrorCode> {
        Gpio::<S>::enable_gpio_input(self.pin_number, P::MODE)?;
        Ok(InputPin {
            pin: self,
            _pull: PhantomData,
        })
    }
}

pub struct OutputPin<'a, S: Syscalls> {
    pin: &'a Pin<S>,
}

impl<'a, S: Syscalls> OutputPin<'a, S> {
    pub fn toggle(&mut self) -> Result<(), ErrorCode> {
        Gpio::<S>::toggle(self.pin.pin_number)
    }
    pub fn set(&mut self) -> Result<(), ErrorCode> {
        Gpio::<S>::write(self.pin.pin_number, GpioState::High)
    }
    pub fn clear(&mut self) -> Result<(), ErrorCode> {
        Gpio::<S>::write(self.pin.pin_number, GpioState::Low)
    }
}

pub struct InputPin<'a, S: Syscalls, P: Pull> {
    pin: &'a Pin<S>,
    _pull: PhantomData<P>,
}

impl<'a, S: Syscalls, P: Pull> InputPin<'a, S, P> {
    pub fn read(&self) -> Option<GpioState> {
        if let Ok(state) = Gpio::<S>::read(self.pin.pin_number) {
            Some(state)
        } else {
            None
        }
    }
}

impl<S: Syscalls> Drop for Pin<S> {
    fn drop(&mut self) {
        let _ = Gpio::<S>::disable(self.pin_number);
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
