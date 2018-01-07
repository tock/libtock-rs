use syscalls::{command, subscribe};

const DRIVER_NUMBER: u32 = 0x00004;
mod gpio_commands {
    pub const SUBSCRIBE_CALLBACK: u32 = 0;
    pub const ENABLE_OUTPUT: u32 = 1;
    pub const SET_HIGH: u32 = 2;
    pub const SET_LOW: u32 = 3;
    pub const TOGGLE: u32 = 4;
    pub const ENABLE_INPUT: u32 = 5;
    pub const READ: u32 = 6;
    pub const ENABLE_INTERRUPT: u32 = 7;
    pub const DISABLE_INTERRUPT: u32 = 8;
    pub const DISABLE: u32 = 9;
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
    fn to_num(self) -> isize {
        match self {
            InputMode::PullNone => 0,
            InputMode::PullUp => 1,
            InputMode::PullDown => 2,
        }
    }
}

impl IrqMode {
    fn to_num(self) -> isize {
        match self {
            IrqMode::EitherEdge => 0,
            IrqMode::RisingEdge => 1,
            IrqMode::FallingEdge => 2,
        }
    }
}

pub struct GpioPinUnitialized {
    number: isize,
}

pub struct GpioPinWrite {
    number: isize,
}

pub struct GpioPinRead {
    number: isize,
}

impl GpioPinUnitialized {
    pub fn new(number: isize) -> GpioPinUnitialized {
        GpioPinUnitialized { number }
    }

    pub fn open_for_write(self) -> Result<GpioPinWrite, &'static str> {
        match unsafe { command(DRIVER_NUMBER, gpio_commands::ENABLE_OUTPUT, self.number, 0) } {
            0 => Ok(GpioPinWrite {
                number: self.number,
            }),
            _ => Err("Could not open pin for writing."),
        }
    }

    pub fn open_for_read(
        self,
        callback: Option<(extern "C" fn(usize, usize, usize, usize), IrqMode)>,
        input_mode: InputMode,
    ) -> Result<GpioPinRead, &'static str> {
        let (callback, irq_mode) = callback.unwrap_or((noop_callback, IrqMode::EitherEdge));
        self.enable_input(input_mode)
            .and_then(|pin| pin.subscribe_callback(callback))
            .and_then(|pin| pin.enable_callback(irq_mode))
    }

    fn subscribe_callback(
        self,
        callback: extern "C" fn(usize, usize, usize, usize),
    ) -> Result<GpioPinUnitialized, &'static str> {
        if unsafe {
            subscribe(
                DRIVER_NUMBER,
                gpio_commands::SUBSCRIBE_CALLBACK,
                callback,
                self.number as usize,
            )
        } == 0
        {
            Ok(self)
        } else {
            Err("Could not subscribe callback.")
        }
    }

    fn enable_input(self, mode: InputMode) -> Result<GpioPinUnitialized, &'static str> {
        if unsafe {
            command(
                DRIVER_NUMBER,
                gpio_commands::ENABLE_INPUT,
                self.number,
                mode.to_num(),
            )
        } == 0
        {
            Ok(self)
        } else {
            Err("Could not enable input.")
        }
    }

    fn enable_callback(self, irq_mode: IrqMode) -> Result<GpioPinRead, &'static str> {
        if unsafe {
            command(
                DRIVER_NUMBER,
                gpio_commands::ENABLE_INTERRUPT,
                self.number,
                irq_mode.to_num(),
            )
        } == 0
        {
            Ok(GpioPinRead {
                number: self.number,
            })
        } else {
            Err("Could not enable callback.")
        }
    }
}

impl GpioPinWrite {
    pub fn set_low(&self) {
        unsafe {
            command(DRIVER_NUMBER, gpio_commands::SET_LOW, self.number, 0);
        }
    }
    pub fn set_high(&self) {
        unsafe {
            command(DRIVER_NUMBER, gpio_commands::SET_HIGH, self.number, 0);
        }
    }
    pub fn toggle(&self) {
        unsafe {
            command(DRIVER_NUMBER, gpio_commands::TOGGLE, self.number, 0);
        }
    }
}

impl GpioPinRead {
    pub fn read(&self) -> bool {
        unsafe { command(DRIVER_NUMBER, gpio_commands::READ, self.number, 0) == 1 }
    }
}

impl Drop for GpioPinWrite {
    fn drop(&mut self) {
        unsafe {
            command(DRIVER_NUMBER, gpio_commands::DISABLE, self.number, 0);
        }
    }
}

impl Drop for GpioPinRead {
    fn drop(&mut self) {
        unsafe {
            command(
                DRIVER_NUMBER,
                gpio_commands::DISABLE_INTERRUPT,
                self.number,
                0,
            );
            command(DRIVER_NUMBER, gpio_commands::DISABLE, self.number, 0);
        }
    }
}

extern "C" fn noop_callback(_: usize, _: usize, _: usize, ptr: usize) {}
