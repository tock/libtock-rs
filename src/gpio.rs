use crate::result::TockResult;
use crate::syscalls;

const DRIVER_NUMBER: usize = 0x00004;
mod gpio_commands {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
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

pub struct GpioPinUnitialized {
    number: usize,
}

pub struct GpioPinWrite {
    number: usize,
}

pub struct GpioPinRead {
    number: usize,
}

impl GpioPinUnitialized {
    pub fn new(number: usize) -> GpioPinUnitialized {
        GpioPinUnitialized { number }
    }

    pub fn open_for_write(self) -> TockResult<GpioPinWrite> {
        syscalls::command(DRIVER_NUMBER, gpio_commands::ENABLE_OUTPUT, self.number, 0)?;
        Ok(GpioPinWrite {
            number: self.number,
        })
    }

    pub fn open_for_read(
        self,
        callback: Option<(extern "C" fn(usize, usize, usize, usize), IrqMode)>,
        input_mode: InputMode,
    ) -> TockResult<GpioPinRead> {
        let (callback, irq_mode) = callback.unwrap_or((noop_callback, IrqMode::EitherEdge));
        self.enable_input(input_mode)
            .and_then(|pin| pin.subscribe_callback(callback))
            .and_then(|pin| pin.enable_callback(irq_mode))
    }

    fn subscribe_callback(
        self,
        callback: extern "C" fn(usize, usize, usize, usize),
    ) -> TockResult<GpioPinUnitialized> {
        syscalls::subscribe_fn(
            DRIVER_NUMBER,
            gpio_commands::SUBSCRIBE_CALLBACK,
            callback,
            self.number,
        )?;
        Ok(self)
    }

    fn enable_input(self, mode: InputMode) -> TockResult<GpioPinUnitialized> {
        syscalls::command(
            DRIVER_NUMBER,
            gpio_commands::ENABLE_INPUT,
            self.number,
            mode.to_num(),
        )?;
        Ok(self)
    }

    fn enable_callback(self, irq_mode: IrqMode) -> TockResult<GpioPinRead> {
        syscalls::command(
            DRIVER_NUMBER,
            gpio_commands::ENABLE_INTERRUPT,
            self.number,
            irq_mode.to_num(),
        )?;
        Ok(GpioPinRead {
            number: self.number,
        })
    }
}

impl GpioPinWrite {
    pub fn set_low(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, gpio_commands::SET_LOW, self.number, 0)?;
        Ok(())
    }
    pub fn set_high(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, gpio_commands::SET_HIGH, self.number, 0)?;
        Ok(())
    }
    pub fn toggle(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, gpio_commands::TOGGLE, self.number, 0)?;
        Ok(())
    }
}

impl GpioPinRead {
    pub fn read(&self) -> bool {
        syscalls::command(DRIVER_NUMBER, gpio_commands::READ, self.number, 0).ok() == Some(1)
    }
}

impl Drop for GpioPinWrite {
    fn drop(&mut self) {
        let _ = syscalls::command(DRIVER_NUMBER, gpio_commands::DISABLE, self.number, 0);
    }
}

impl Drop for GpioPinRead {
    fn drop(&mut self) {
        let _ = syscalls::command(
            DRIVER_NUMBER,
            gpio_commands::DISABLE_INTERRUPT,
            self.number,
            0,
        );
        let _ = syscalls::command(DRIVER_NUMBER, gpio_commands::DISABLE, self.number, 0);
    }
}

extern "C" fn noop_callback(_: usize, _: usize, _: usize, _: usize) {}
