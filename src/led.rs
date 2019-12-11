use crate::result::TockResult;
use crate::syscalls::command;

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

pub fn count() -> TockResult<usize> {
    command(DRIVER_NUMBER, command_nr::COUNT, 0, 0).map_err(Into::into)
}

pub fn get(led_num: usize) -> Option<Led> {
    if led_num < count().ok().unwrap() {
        Some(Led { led_num })
    } else {
        None
    }
}

pub fn all() -> LedIter {
    LedIter {
        curr_led: 0,
        led_count: count().ok().unwrap(),
    }
}

impl Led {
    pub fn set_state(&self, state: bool) -> TockResult<()> {
        if state {
            self.on()
        } else {
            self.off()
        }
    }

    pub fn on(&self) -> TockResult<()> {
        command(DRIVER_NUMBER, command_nr::ON, self.led_num, 0)?;
        Ok(())
    }

    pub fn off(&self) -> TockResult<()> {
        command(DRIVER_NUMBER, command_nr::OFF, self.led_num, 0)?;
        Ok(())
    }

    pub fn toggle(&self) -> TockResult<()> {
        command(DRIVER_NUMBER, command_nr::TOGGLE, self.led_num, 0)?;
        Ok(())
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
