use syscalls::command;
const DRIVER_NUMBER: u32 = 0x00004;
const ENABLE_OUTPUT: u32 = 1;
const SET_HIGH: u32 = 2;
const SET_LOW: u32 = 3;
const TOGGLE: u32 = 4;
const DISABLE: u32 = 9;

pub struct GpioPinUnitialized {
    number: isize,
}

pub struct GpioPinWrite {
    number: isize,
}

impl GpioPinUnitialized {
    pub fn new(number: isize)-> GpioPinUnitialized{
        GpioPinUnitialized { number }
    }

    pub fn open_for_write(self) -> Result<GpioPinWrite, &'static str> {
        match unsafe { command(DRIVER_NUMBER, ENABLE_OUTPUT, self.number) } {
            0 => Ok(GpioPinWrite { number: self.number }),
            _ => Err("Could not open pin for writing."),
        }
    }
}

impl GpioPinWrite {
    pub fn set_low(&self) {
        unsafe { command(DRIVER_NUMBER, SET_LOW, self.number); }
    }
    pub fn set_high(&self) {
        unsafe { command(DRIVER_NUMBER, SET_HIGH, self.number); }
    }
    pub fn toggle(&self) {
        unsafe { command(DRIVER_NUMBER, TOGGLE, self.number); }
    }
}

impl Drop for GpioPinWrite {
    fn drop(&mut self) {
        unsafe { command(DRIVER_NUMBER, DISABLE, self.number); }
    }
}
