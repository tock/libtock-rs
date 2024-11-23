//! Fake implementation of the Alarm API.
//!
//! Supports frequency and set_relative.
//! Will schedule the upcall immediately.

use core::cell::Cell;
use core::num::Wrapping;
use libtock_platform::{CommandReturn, ErrorCode};

use crate::{DriverInfo, DriverShareRef};

pub struct Alarm {
    frequency_hz: u32,
    now: Cell<Wrapping<u32>>,
    share_ref: DriverShareRef,
}

impl Alarm {
    pub fn new(frequency_hz: u32) -> std::rc::Rc<Alarm> {
        std::rc::Rc::new(Alarm {
            frequency_hz,
            now: Cell::new(Wrapping(0)),
            share_ref: Default::default(),
        })
    }
}

impl crate::fake::SyscallDriver for Alarm {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_number: u32, argument0: u32, _argument1: u32) -> CommandReturn {
        match command_number {
            command::FREQUENCY => crate::command_return::success_u32(self.frequency_hz),
            command::SET_RELATIVE => {
                // We're not actually sleeping, just ticking the timer.
                // The semantics of sleeping aren't clear,
                // so we're assuming that all future times are equal,
                // and waking immediately.
                let relative = argument0;
                let wake = self.now.get() + Wrapping(relative);
                self.share_ref
                    .schedule_upcall(subscribe::CALLBACK, (wake.0, 0, 0))
                    .expect("schedule_upcall failed");
                self.now.set(wake);
                crate::command_return::success_u32(wake.0)
            }
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x0;

// Command IDs
#[allow(unused)]
pub mod command {
    pub const EXISTS: u32 = 0;
    pub const FREQUENCY: u32 = 1;
    pub const TIME: u32 = 2;
    pub const STOP: u32 = 3;

    pub const SET_RELATIVE: u32 = 5;
    pub const SET_ABSOLUTE: u32 = 6;
}

#[allow(unused)]
pub mod subscribe {
    pub const CALLBACK: u32 = 0;
}
