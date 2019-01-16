use crate::gpio::GpioPinWrite;

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

    pub fn write_bits(&self, values: &[bool]) {
        for i in values {
            self.push_bit(*i);
        }
        self.display();
    }

    fn push_bit(&self, value: bool) {
        if value {
            self.data_pin.set_high();
        } else {
            self.data_pin.set_low();
        }
        self.clock_pin.set_high();
        self.clock_pin.set_low();
    }

    fn display(&self) {
        self.latch_pin.set_high();
        self.latch_pin.set_low();
    }
}
