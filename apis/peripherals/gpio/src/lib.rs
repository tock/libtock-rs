#![no_std]

use core::marker::PhantomData;

use libtock_platform::{
    share::Handle, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

/// The GPIO driver.
///
/// # Example
/// ```ignore
/// use libtock::gpio;
///
/// // Set pin to high.
/// let pin = gpio::Gpio::get_pin(0).unwrap().make_output().unwrap();
/// let _ = pin.set();
/// ```

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GpioState {
    Low = 0,
    High = 1,
}

pub enum PinInterruptEdge {
    Either = 0,
    Rising = 1,
    Falling = 2,
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

pub struct Gpio<S: Syscalls>(S);

impl<S: Syscalls> Gpio<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    pub fn count() -> Result<u32, ErrorCode> {
        S::command(DRIVER_NUM, GPIO_COUNT, 0, 0).to_result()
    }

    pub fn get_pin(pin: u32) -> Result<Pin<S>, ErrorCode> {
        Self::disable(pin)?;
        Ok(Pin {
            pin_number: pin,
            _syscalls: PhantomData,
        })
    }

    /// Register an interrupt listener
    ///
    /// There can be only one single listener registered at a time.
    /// Each time this function is used, it will replace the
    /// previously registered listener.
    pub fn register_listener<'share, F: Fn(u32, GpioState)>(
        listener: &'share GpioInterruptListener<F>,
        subscribe: Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Unregister the interrupt listener
    ///
    /// This function may be used even if there was no
    /// previously registered listener.
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }
}

/// A wrapper around a closure to be registered and called when
/// a gpio interrupt occurs.
///
/// ```ignore
/// let listener = GpioInterruptListener(|gpio, interrupt_edge| {
///     // make use of the button's state
/// });
/// ```
pub struct GpioInterruptListener<F: Fn(u32, GpioState)>(pub F);

impl<F: Fn(u32, GpioState)> Upcall<OneId<DRIVER_NUM, 0>> for GpioInterruptListener<F> {
    fn upcall(&self, gpio_index: u32, value: u32, _arg2: u32) {
        self.0(gpio_index, value.into())
    }
}

impl From<u32> for GpioState {
    fn from(original: u32) -> GpioState {
        match original {
            0 => GpioState::Low,
            _ => GpioState::High,
        }
    }
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
    pub fn read(&self) -> Result<GpioState, ErrorCode> {
        Gpio::<S>::read(self.pin.pin_number)
    }

    pub fn enable_interrupts(&self, edge: PinInterruptEdge) -> Result<(), ErrorCode> {
        Gpio::<S>::enable_interrupts(self.pin.pin_number, edge)
    }

    pub fn disable_interrupts(&self) -> Result<(), ErrorCode> {
        Gpio::<S>::disable_interrupts(self.pin.pin_number)
    }
}

impl<S: Syscalls> Drop for OutputPin<'_, S> {
    fn drop(&mut self) {
        let _ = Gpio::<S>::disable(self.pin.pin_number);
    }
}

impl<S: Syscalls, P: Pull> Drop for InputPin<'_, S, P> {
    fn drop(&mut self) {
        let _ = Gpio::<S>::disable(self.pin.pin_number);
    }
}

// -----------------------------------------------------------------------------
// Implementation details below
// -----------------------------------------------------------------------------

impl<S: Syscalls> Gpio<S> {
    fn enable_gpio_output(pin: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, GPIO_ENABLE_OUTPUT, pin, 0).to_result()
    }

    fn enable_gpio_input(pin: u32, mode: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, GPIO_ENABLE_INPUT, pin, mode).to_result()
    }

    fn write(pin: u32, state: GpioState) -> Result<(), ErrorCode> {
        let action = match state {
            GpioState::Low => GPIO_CLEAR,
            _ => GPIO_SET,
        };
        S::command(DRIVER_NUM, action, pin, 0).to_result()
    }

    fn read(pin: u32) -> Result<GpioState, ErrorCode> {
        let pin_state: u32 = S::command(DRIVER_NUM, GPIO_READ_INPUT, pin, 0).to_result()?;
        Ok(pin_state.into())
    }

    fn toggle(pin: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, GPIO_TOGGLE, pin, 0).to_result()
    }

    fn disable(pin: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, GPIO_DISABLE, pin, 0).to_result()
    }

    fn enable_interrupts(pin: u32, edge: PinInterruptEdge) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, GPIO_ENABLE_INTERRUPTS, pin, edge as u32).to_result()
    }

    fn disable_interrupts(pin: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, GPIO_DISABLE_INTERRUPTS, pin, 0).to_result()
    }
}

#[cfg(feature = "rust_embedded")]
impl<'a, S: Syscalls> embedded_hal::digital::ErrorType for OutputPin<'a, S> {
    type Error = ErrorCode;
}

#[cfg(feature = "rust_embedded")]
impl<'a, S: Syscalls> embedded_hal::digital::OutputPin for OutputPin<'a, S> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.clear()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set()
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
