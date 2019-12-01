use crate::gpio::GpioWrite;
use crate::result::TockResult;

pub struct ShiftRegister<'a> {
    data_pin: &'a GpioWrite<'a>,
    clock_pin: &'a GpioWrite<'a>,
    latch_pin: &'a GpioWrite<'a>,
}

impl<'a> ShiftRegister<'a> {
    pub fn new(
        data_pin: &'a GpioWrite<'a>,
        clock_pin: &'a GpioWrite<'a>,
        latch_pin: &'a GpioWrite<'a>,
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
