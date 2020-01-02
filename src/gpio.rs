use crate::result::OtherError;
use crate::result::TockError;
use crate::result::TockResult;
use crate::syscalls;
use core::marker::PhantomData;

const DRIVER_NUMBER: usize = 0x00004;
mod command_nr {
    pub const NUMBER_PINS: usize = 0;
    pub const ENABLE_OUTPUT: usize = 1;
    pub const SET_HIGH: usize = 2;
    pub const SET_LOW: usize = 3;
    pub const TOGGLE: usize = 4;
    pub const ENABLE_INPUT: usize = 5;
    pub const READ: usize = 6;
    pub const ENABLE_INTERRUPT: usize = 7;
    pub const DISABLE_INTERRUPT: usize = 8;
    pub const DISABLE: usize = 9;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

pub struct GpioDriver {
    pub(crate) _unconstructible: (),
}

impl GpioDriver {
    pub fn all_pins<'a>(&'a self) -> TockResult<GpioIter<'a>> {
        let number = self.number_of_pins()?;
        Ok(GpioIter {
            curr_gpio: 0,
            gpio_count: number,
            phantom: PhantomData,
        })
    }

    pub fn pin<'a>(&'a self, pin: usize) -> TockResult<GpioPinUnitialized<'a>> {
        let number = self.number_of_pins()?;
        if pin < number {
            Ok(GpioPinUnitialized {
                number: pin,
                phantom: PhantomData,
            })
        } else {
            Err(TockError::Other(OtherError::NotEnoughGpioPins))
        }
    }

    fn number_of_pins(&self) -> TockResult<usize> {
        syscalls::command(DRIVER_NUMBER, command_nr::NUMBER_PINS, 0, 0).map_err(Into::into)
    }
}

#[derive(Copy, Clone)]
pub struct GpioIter<'a> {
    curr_gpio: usize,
    gpio_count: usize,
    phantom: PhantomData<&'a mut ()>,
}

impl<'a> Iterator for GpioIter<'a> {
    type Item = GpioPinUnitialized<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_gpio < self.gpio_count {
            let item = GpioPinUnitialized {
                number: self.curr_gpio,
                phantom: PhantomData,
            };
            self.curr_gpio += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub enum InputMode {
    PullUp,
    PullDown,
    PullNone,
}

pub enum IrqMode {
    EitherEdge,
    RisingEdge,
    FallingEdge,
}

impl InputMode {
    fn to_num(&self) -> usize {
        match self {
            InputMode::PullNone => 0,
            InputMode::PullUp => 1,
            InputMode::PullDown => 2,
        }
    }
}

impl IrqMode {
    fn to_num(&self) -> usize {
        match self {
            IrqMode::EitherEdge => 0,
            IrqMode::RisingEdge => 1,
            IrqMode::FallingEdge => 2,
        }
    }
}

pub struct GpioPinUnitialized<'a> {
    number: usize,
    phantom: PhantomData<&'a mut ()>,
}

pub struct GpioPinWrite<'a> {
    number: usize,
    phantom: PhantomData<&'a mut ()>,
}

pub struct GpioPinRead<'a> {
    number: usize,
    phantom: PhantomData<&'a mut ()>,
}

impl<'a> GpioPinUnitialized<'a> {
    pub fn open_for_write(self) -> TockResult<GpioPinWrite<'a>> {
        syscalls::command(DRIVER_NUMBER, command_nr::ENABLE_OUTPUT, self.number, 0)?;
        Ok(GpioPinWrite {
            number: self.number,
            phantom: PhantomData,
        })
    }

    pub fn open_for_read(
        self,
        callback: Option<(extern "C" fn(usize, usize, usize, usize), IrqMode)>,
        input_mode: InputMode,
    ) -> TockResult<GpioPinRead<'a>> {
        let (callback, irq_mode) = callback.unwrap_or((noop_callback, IrqMode::EitherEdge));
        self.enable_input(input_mode)
            .and_then(|pin| pin.subscribe_callback(callback))
            .and_then(move |pin| pin.enable_callback(irq_mode))
    }

    fn subscribe_callback(
        self,
        callback: extern "C" fn(usize, usize, usize, usize),
    ) -> TockResult<GpioPinUnitialized<'a>> {
        syscalls::subscribe_fn(
            DRIVER_NUMBER,
            subscribe_nr::SUBSCRIBE_CALLBACK,
            callback,
            self.number,
        )?;
        Ok(self)
    }

    fn enable_input(self, mode: InputMode) -> TockResult<GpioPinUnitialized<'a>> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::ENABLE_INPUT,
            self.number,
            mode.to_num(),
        )?;
        Ok(self)
    }

    fn enable_callback(self, irq_mode: IrqMode) -> TockResult<GpioPinRead<'a>> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::ENABLE_INTERRUPT,
            self.number,
            irq_mode.to_num(),
        )?;
        Ok(GpioPinRead {
            number: self.number,
            phantom: PhantomData,
        })
    }
}

impl<'a> GpioPinWrite<'a> {
    pub fn set_low(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::SET_LOW, self.number, 0)?;
        Ok(())
    }
    pub fn set_high(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::SET_HIGH, self.number, 0)?;
        Ok(())
    }
    pub fn toggle(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::TOGGLE, self.number, 0)?;
        Ok(())
    }
}

impl<'a> GpioPinRead<'a> {
    pub fn read(&'a self) -> bool {
        syscalls::command(DRIVER_NUMBER, command_nr::READ, self.number, 0).ok() == Some(1)
    }
}

impl<'a> Drop for GpioPinWrite<'a> {
    fn drop(&mut self) {
        let _ = syscalls::command(DRIVER_NUMBER, command_nr::DISABLE, self.number, 0);
    }
}

impl<'a> Drop for GpioPinRead<'a> {
    fn drop(&mut self) {
        let _ = syscalls::command(DRIVER_NUMBER, command_nr::DISABLE_INTERRUPT, self.number, 0);
        let _ = syscalls::command(DRIVER_NUMBER, command_nr::DISABLE, self.number, 0);
    }
}

extern "C" fn noop_callback(_: usize, _: usize, _: usize, _: usize) {}
