use crate::result::TockResult;
use crate::syscalls::command;
use core::marker::PhantomData;

const DRIVER_NUMBER: usize = 0x00002;

mod command_nr {
    pub const COUNT: usize = 0;
    pub const ON: usize = 1;
    pub const OFF: usize = 2;
    pub const TOGGLE: usize = 3;
}

pub struct LedDriver {
    pub(crate) _unconstructible: (),
}

pub struct Led<'a> {
    led_num: usize,
    phantom: PhantomData<&'a mut ()>,
}

impl LedDriver {
    pub fn get(&mut self, led_num: usize) -> Option<Led> {
        if led_num < self.count().ok().unwrap() {
            Some(Led {
                led_num,
                phantom: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn count(&self) -> TockResult<usize> {
        command(DRIVER_NUMBER, command_nr::COUNT, 0, 0).map_err(Into::into)
    }

    pub fn all(&mut self) -> LedIter {
        LedIter {
            curr_led: 0,
            led_count: self.count().unwrap_or(0),
            phantom: PhantomData,
        }
    }
}

/// Returns an iterator over all available LEDs. If the LED driver is not
/// present, the iterator will be empty.

impl<'a> Led<'a> {
    pub fn set_state(&mut self, state: bool) -> TockResult<()> {
        if state {
            self.on()
        } else {
            self.off()
        }
    }

    pub fn on(&mut self) -> TockResult<()> {
        command(DRIVER_NUMBER, command_nr::ON, self.led_num, 0)?;
        Ok(())
    }

    pub fn off(&mut self) -> TockResult<()> {
        command(DRIVER_NUMBER, command_nr::OFF, self.led_num, 0)?;
        Ok(())
    }

    pub fn toggle(&mut self) -> TockResult<()> {
        command(DRIVER_NUMBER, command_nr::TOGGLE, self.led_num, 0)?;
        Ok(())
    }

    pub fn number(&self) -> usize {
        self.led_num
    }
}

#[derive(Copy, Clone)]
pub struct LedIter<'a> {
    curr_led: usize,
    led_count: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for LedIter<'a> {
    type Item = Led<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_led < self.led_count {
            let item = Led {
                led_num: self.curr_led,
                phantom: PhantomData,
            };
            self.curr_led += 1;
            Some(item)
        } else {
            None
        }
    }
}
