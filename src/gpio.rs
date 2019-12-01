use crate::callback::CallbackSubscription;
use crate::callback::Consumer;
use crate::result::OtherError;
use crate::result::TockResult;
use crate::syscalls;
use core::marker::PhantomData;

const DRIVER_NUMBER: usize = 0x00004;

mod command_nr {
    pub const COUNT: usize = 0;
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

#[non_exhaustive]
pub struct GpioDriverFactory;

impl GpioDriverFactory {
    pub fn init_driver(&mut self) -> TockResult<GpioDriver> {
        let driver = GpioDriver {
            num_gpios: syscalls::command(DRIVER_NUMBER, command_nr::COUNT, 0, 0)?,
            lifetime: PhantomData,
        };
        Ok(driver)
    }
}

pub struct GpioDriver<'a> {
    num_gpios: usize,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> GpioDriver<'a> {
    pub fn num_gpios(&self) -> usize {
        self.num_gpios
    }

    pub fn gpios(&mut self) -> Gpios {
        Gpios {
            num_gpios: self.num_gpios(),
            curr_gpio: 0,
            lifetime: PhantomData,
        }
    }

    pub fn subscribe<CB: Fn(usize, GpioState)>(
        &self,
        callback: &'a mut CB,
    ) -> TockResult<CallbackSubscription> {
        syscalls::subscribe::<GpioEventConsumer, _>(
            DRIVER_NUMBER,
            subscribe_nr::SUBSCRIBE_CALLBACK,
            callback,
        )
        .map_err(Into::into)
    }
}

struct GpioEventConsumer;

impl<CB: Fn(usize, GpioState)> Consumer<CB> for GpioEventConsumer {
    fn consume(callback: &mut CB, gpio_num: usize, gpio_state: usize, _: usize) {
        let gpio_state = match gpio_state {
            0 => GpioState::Low,
            1 => GpioState::High,
            _ => return,
        };
        callback(gpio_num, gpio_state);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GpioState {
    Low,
    High,
}

impl From<GpioState> for bool {
    fn from(gpio_state: GpioState) -> Self {
        match gpio_state {
            GpioState::Low => false,
            GpioState::High => true,
        }
    }
}

impl From<bool> for GpioState {
    fn from(from_value: bool) -> Self {
        if from_value {
            GpioState::Low
        } else {
            GpioState::High
        }
    }
}

pub struct Gpios<'a> {
    num_gpios: usize,
    curr_gpio: usize,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> Iterator for Gpios<'a> {
    type Item = Gpio<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_gpio < self.num_gpios {
            let item = Gpio {
                gpio_num: self.curr_gpio,
                lifetime: PhantomData,
            };
            self.curr_gpio += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct Gpio<'a> {
    gpio_num: usize,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> Gpio<'a> {
    pub fn enable_output(&mut self) -> TockResult<GpioWrite> {
        syscalls::command(DRIVER_NUMBER, command_nr::ENABLE_OUTPUT, self.gpio_num, 0)?;
        let gpio_write = GpioWrite {
            gpio_num: self.gpio_num,
            lifetime: PhantomData,
        };
        Ok(gpio_write)
    }

    pub fn enable_input(&mut self, resistor_mode: ResistorMode) -> TockResult<GpioRead> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::ENABLE_INPUT,
            self.gpio_num,
            resistor_mode as usize,
        )?;
        let gpio_read = GpioRead {
            gpio_num: self.gpio_num,
            lifetime: PhantomData,
        };
        Ok(gpio_read)
    }
}

pub struct GpioWrite<'a> {
    gpio_num: usize,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> GpioWrite<'a> {
    pub fn gpio_num(&self) -> usize {
        self.gpio_num
    }

    pub fn set(&self, state: impl Into<GpioState>) -> TockResult<()> {
        match state.into() {
            GpioState::Low => self.set_low(),
            GpioState::High => self.set_high(),
        }
    }

    pub fn set_low(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::SET_LOW, self.gpio_num, 0)?;
        Ok(())
    }

    pub fn set_high(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::SET_HIGH, self.gpio_num, 0)?;
        Ok(())
    }

    pub fn toggle(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::TOGGLE, self.gpio_num, 0)?;
        Ok(())
    }
}

impl<'a> Drop for GpioWrite<'a> {
    fn drop(&mut self) {
        let _ = syscalls::command(DRIVER_NUMBER, command_nr::DISABLE, self.gpio_num, 0);
    }
}

pub struct GpioRead<'a> {
    gpio_num: usize,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> GpioRead<'a> {
    pub fn gpio_num(&self) -> usize {
        self.gpio_num
    }

    pub fn read(&self) -> TockResult<GpioState> {
        let button_state = syscalls::command(DRIVER_NUMBER, command_nr::READ, self.gpio_num, 0)?;
        match button_state {
            0 => Ok(GpioState::Low),
            1 => Ok(GpioState::High),
            _ => Err(OtherError::GpioDriverInvalidState.into()),
        }
    }

    pub fn enable_interrupt(&self, trigger_type: TriggerType) -> TockResult<()> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::ENABLE_INTERRUPT,
            self.gpio_num,
            trigger_type as usize,
        )?;
        Ok(())
    }

    pub fn disable_interrupt(&self, trigger_type: TriggerType) -> TockResult<()> {
        syscalls::command(
            DRIVER_NUMBER,
            command_nr::DISABLE_INTERRUPT,
            self.gpio_num,
            trigger_type as usize,
        )?;
        Ok(())
    }
}

impl<'a> Drop for GpioRead<'a> {
    fn drop(&mut self) {
        let _ = syscalls::command(DRIVER_NUMBER, command_nr::DISABLE, self.gpio_num, 0);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResistorMode {
    PullNone = 0,
    PullUp = 1,
    PullDown = 2,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TriggerType {
    EitherEdge = 0,
    RisingEdge = 1,
    FallingEdge = 2,
}
