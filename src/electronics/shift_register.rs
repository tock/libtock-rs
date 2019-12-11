use crate::gpio::GpioPinWrite;
use crate::result::TockResult;

pub struct ShiftRegister {
    data_pin: GpioPinWrite,
    clock_pin: GpioPinWrite,
    latch_pin: GpioPinWrite,
}

impl ShiftRegister {
    pub fn new(
        data_pin: GpioPinWrite,
        clock_pin: GpioPinWrite,
        latch_pin: GpioPinWrite,
    ) -> ShiftRegister {
        ShiftRegister {
            data_pin,
            clock_pin,
            latch_pin,
        }
    }

    pub fn write_bits(&self, values: &[bool]) -> TockResult<()> {
        for i in values {
            self.push_bit(*i)?;
        }
        self.display()
    }

    fn push_bit(&self, value: bool) -> TockResult<()> {
        if value {
            self.data_pin.set_high()
        } else {
            self.data_pin.set_low()
        }?;
        self.clock_pin.set_high()?;
        self.clock_pin.set_low()
    }

    fn display(&self) -> TockResult<()> {
        self.latch_pin.set_high()?;
        self.latch_pin.set_low()
    }
}
