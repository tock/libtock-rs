use crate::gpio::GpioPinWrite;
use crate::result::TockResult;

pub struct ShiftRegister<'a> {
    data_pin: GpioPinWrite<'a>,
    clock_pin: GpioPinWrite<'a>,
    latch_pin: GpioPinWrite<'a>,
}

impl<'a> ShiftRegister<'a> {
    pub fn new(
        data_pin: GpioPinWrite<'a>,
        clock_pin: GpioPinWrite<'a>,
        latch_pin: GpioPinWrite<'a>,
    ) -> ShiftRegister<'a> {
        ShiftRegister {
            data_pin,
            clock_pin,
            latch_pin,
        }
    }

    pub fn write_bits(&mut self, values: &[bool]) -> TockResult<()> {
        for i in values {
            self.push_bit(*i)?;
        }
        self.display()
    }

    fn push_bit(&mut self, value: bool) -> TockResult<()> {
        if value {
            self.data_pin.set_high()
        } else {
            self.data_pin.set_low()
        }?;
        self.clock_pin.set_high()?;
        self.clock_pin.set_low()
    }

    fn display(&mut self) -> TockResult<()> {
        self.latch_pin.set_high()?;
        self.latch_pin.set_low()
    }
}
