use led;

pub struct ShiftRegister {
    data_pin: u32,
    clock_pin: u32,
    latch_pin: u32,
}

impl ShiftRegister {
    pub fn new(data_pin: u32, clock_pin: u32, latch_pin: u32) -> ShiftRegister {
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
            led::on(self.data_pin)
        } else {
            led::off(self.data_pin)
        }
        led::on(self.clock_pin);
        led::off(self.clock_pin);
    }

    fn display(&self) {
        led::on(self.latch_pin);
        led::off(self.latch_pin);
    }
}
