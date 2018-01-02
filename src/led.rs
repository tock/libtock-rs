use syscalls::command;

const DRIVER_NUMBER: u32 = 0x00002;

mod command_nr {
    pub const COUNT: u32 = 0;
    pub const ON: u32 = 1;
    pub const OFF: u32 = 2;
    pub const TOGGLE: u32 = 3;
}

pub struct Led {
    led_num: isize,
}

pub fn count() -> isize {
    unsafe { command(DRIVER_NUMBER, command_nr::COUNT, 0) }
}

pub fn get(led_num: isize) -> Option<Led> {
    if led_num >= 0 && led_num < count() {
        Some(Led { led_num })
    } else {
        None
    }
}

pub fn all() -> LedIter {
    LedIter {
        curr_led: 0,
        led_count: count(),
    }
}

impl Led {
    pub fn set_state(&self, state: bool) {
        if state {
            self.on()
        } else {
            self.off()
        }
    }

    pub fn on(&self) {
        unsafe {
            command(DRIVER_NUMBER, command_nr::ON, self.led_num);
        }
    }

    pub fn off(&self) {
        unsafe {
            command(DRIVER_NUMBER, command_nr::OFF, self.led_num);
        }
    }

    pub fn toggle(&self) {
        unsafe {
            command(DRIVER_NUMBER, command_nr::TOGGLE, self.led_num);
        }
    }
}

#[derive(Copy, Clone)]
pub struct LedIter {
    curr_led: isize,
    led_count: isize,
}

impl Iterator for LedIter {
    type Item = Led;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_led < self.led_count {
            let item = Led {
                led_num: self.curr_led,
            };
            self.curr_led += 1;
            Some(item)
        } else {
            None
        }
    }
}
