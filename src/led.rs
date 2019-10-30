use crate::syscalls::{command0, command1};

const DRIVER_NUMBER: usize = 0x00002;

mod command_nr {
    pub const COUNT: usize = 0;
    pub const ON: usize = 1;
    pub const OFF: usize = 2;
    pub const TOGGLE: usize = 3;
}

pub struct Led {
    led_num: usize,
}

pub fn count() -> isize {
    unsafe { command0(DRIVER_NUMBER, command_nr::COUNT) }
}

pub fn get(led_num: isize) -> Option<Led> {
    if led_num >= 0 && led_num < count() {
        Some(Led {
            led_num: led_num as usize,
        })
    } else {
        None
    }
}

pub fn all() -> LedIter {
    LedIter {
        curr_led: 0,
        led_count: count() as usize,
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
            command1(DRIVER_NUMBER, command_nr::ON, self.led_num);
        }
    }

    pub fn off(&self) {
        unsafe {
            command1(DRIVER_NUMBER, command_nr::OFF, self.led_num);
        }
    }

    pub fn toggle(&self) {
        unsafe {
            command1(DRIVER_NUMBER, command_nr::TOGGLE, self.led_num);
        }
    }
}

#[derive(Copy, Clone)]
pub struct LedIter {
    curr_led: usize,
    led_count: usize,
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
