//! Fake implementation of the Ambient Light API, documented here:
//! https://github.com/tock/tock/blob/master/doc/syscalls/60002_luminance.md
//!
//! Like the real API, `AmbientLight` controls a fake ambient light sensor. It provides
//! a function `set_value` used to immediately call an upcall with a intensity value read by the sensor
//! and a function 'set_value_sync' used to call the upcall when the read command is received.

use crate::{DriverInfo, DriverShareRef};
use libtock_platform::{CommandReturn, ErrorCode};
use std::cell::Cell;

// The `upcall_on_command` field is set to Some(value) if an upcall(with value as its argument) should be called when read command is received,
// or None otherwise. It was needed for testing `read_sync` library function which simulates a synchronous temperature read,
// because it was impossible to schedule an upcall during the `synchronous` read in other ways.
pub struct AmbientLight {
    busy: Cell<bool>,
    upcall_on_command: Cell<Option<u32>>,
    share_ref: DriverShareRef,
}

impl AmbientLight {
    pub fn new() -> std::rc::Rc<AmbientLight> {
        std::rc::Rc::new(AmbientLight {
            busy: Cell::new(false),
            upcall_on_command: Cell::new(None),
            share_ref: Default::default(),
        })
    }

    pub fn is_busy(&self) -> bool {
        self.busy.get()
    }
    pub fn set_value(&self, value: u32) {
        if self.busy.get() {
            self.share_ref
                .schedule_upcall(0, (value, 0, 0))
                .expect("Unable to schedule upcall");
            self.busy.set(false);
        }
    }
    pub fn set_value_sync(&self, value: u32) {
        self.upcall_on_command.set(Some(value));
    }
}

impl crate::fake::SyscallDriver for AmbientLight {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_id: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_id {
            EXISTS => crate::command_return::success(),

            READ_INTENSITY => {
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.busy.set(true);
                if let Some(val) = self.upcall_on_command.take() {
                    self.set_value(val);
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

const DRIVER_NUM: u32 = 0x60002;

// Command IDs

const EXISTS: u32 = 0;
const READ_INTENSITY: u32 = 1;
