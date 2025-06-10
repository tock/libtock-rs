//! Fake implementation of the Adc API, documented here:
//!
//! Like the real API, `Adc` controls a fake Adc sensor. It provides
//! a function `set_value` used to immediately call an upcall with a Adc value read by the sensor
//! and a function 'set_value_sync' used to call the upcall when the read command is received.

use crate::{DriverInfo, DriverShareRef};
use libtock_platform::{CommandReturn, ErrorCode};
use std::cell::Cell;

// The `upcall_on_command` field is set to Some(value) if an upcall(with value as its argument) should be called when read command is received,
// or None otherwise. It was needed for testing `read_sync` library function which simulates a synchronous Adc read,
// because it was impossible to schedule an upcall during the `synchronous` read in other ways.
pub struct Adc {
    busy: Cell<bool>,
    upcall_on_command: [Cell<Option<i32>>; 3],
    share_ref: DriverShareRef,
}

impl Adc {
    pub fn new() -> std::rc::Rc<Adc> {
        std::rc::Rc::new(Adc {
            busy: Cell::new(false),
            upcall_on_command: [Cell::new(None), Cell::new(None), Cell::new(None)],
            share_ref: Default::default(),
        })
    }

    pub fn is_busy(&self) -> bool {
        self.busy.get()
    }
    pub fn set_value(&self, channel: i32, sample: i32) {
        if self.busy.get() {
            self.share_ref
                .schedule_upcall(0, (0, channel as u32, sample as u32))
                .expect("Unable to schedule upcall");
            self.busy.set(false);
        }
    }
    pub fn set_value_sync(&self, channel: i32, sample: i32) {
        self.upcall_on_command[0].set(Some(0));
        self.upcall_on_command[1].set(Some(channel));
        self.upcall_on_command[2].set(Some(sample));
    }
}

impl crate::fake::SyscallDriver for Adc {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_id: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_id {
            EXISTS => crate::command_return::success_u32(1),

            SINGLE_SAMPLE => {
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.busy.set(true);
                if let (Some(0), Some(channel), Some(sample)) = (
                    self.upcall_on_command[0].take(),
                    self.upcall_on_command[1].take(),
                    self.upcall_on_command[2].take(),
                ) {
                    self.set_value(channel, sample);
                }
                crate::command_return::success()
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

const DRIVER_NUM: u32 = 0x5;

// Command IDs

const EXISTS: u32 = 0;
const SINGLE_SAMPLE: u32 = 1;
