use led::Led;

// TODO: Create Gpio abstraction
pub struct ShiftRegister {
    data_pin: Led,
    clock_pin: Led,
    latch_pin: Led,
}

impl ShiftRegister {
    pub fn new(data_pin: Led, clock_pin: Led, latch_pin: Led) -> ShiftRegister {
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
        self.data_pin.set_state(value);
        self.clock_pin.on();
        self.clock_pin.off();
    }

    fn display(&self) {
        self.latch_pin.on();
        self.latch_pin.off();
    }
}
