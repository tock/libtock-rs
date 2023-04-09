//! Fake implementation of the Proximity API.
//!
//! Like the real API, `Proximity` controls a fake proximity sensor.
//! It provides a function `set_value` used to immediately simulate a sensor reading,
//! and a function `set_value_sync` used to simulate a sensor reading when one
//! of the read commands is received.

use std::cell::Cell;

use crate::{DriverInfo, DriverShareRef};
use libtock_platform::{CommandReturn, ErrorCode};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProximityCommand {
    ReadProximity = 1,
    ReadProximityOnInterrupt = 2,
    NoCommand = 3,
}
#[derive(Default, Clone, Copy)]
pub struct Thresholds {
    lower: u8,
    upper: u8,
}

// The `upcall_on_command` field is set to Some(value) if an upcall(with value as its argument) should be called when a read command is received,
// or None otherwise. It was needed for testing `read_sync` library functions which simulates a synchronous sensor read,
// because it was impossible to schedule an upcall during the `synchronous` read in other ways.
pub struct Proximity {
    current_command: Cell<ProximityCommand>,
    thresholds: Cell<Thresholds>,
    upcall_on_command: Cell<Option<u8>>,
    share_ref: DriverShareRef,
}

impl Proximity {
    pub fn new() -> std::rc::Rc<Proximity> {
        std::rc::Rc::new(Proximity {
            current_command: Cell::new(ProximityCommand::NoCommand),
            thresholds: Cell::new(Thresholds { lower: 0, upper: 0 }),
            upcall_on_command: Cell::new(None),
            share_ref: Default::default(),
        })
    }

    pub fn set_value(&self, value: u8) {
        //should not schedule an upcall if no reading was initiated or interrupt conditions are not true
        match self.current_command.get() {
            ProximityCommand::NoCommand => return,
            ProximityCommand::ReadProximityOnInterrupt => {
                if value >= self.thresholds.get().lower && value <= self.thresholds.get().upper {
                    return;
                }
            }
            ProximityCommand::ReadProximity => {}
        }

        self.share_ref
            .schedule_upcall(0, (value as u32, 0, 0))
            .expect("Unable to schedule upcall");
        self.current_command.set(ProximityCommand::NoCommand);
        self.upcall_on_command.set(None);
    }

    pub fn set_value_sync(&self, value: u8) {
        self.upcall_on_command.set(Some(value));
    }
}

impl crate::fake::SyscallDriver for Proximity {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_id: u32, argument0: u32, argument1: u32) -> CommandReturn {
        match command_id {
            EXISTS => crate::command_return::success(),

            READ => {
                if self.current_command.get() != ProximityCommand::NoCommand {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.current_command.set(ProximityCommand::ReadProximity);
                if let Some(val) = self.upcall_on_command.get() {
                    self.set_value(val);
                }
                crate::command_return::success()
            }
            READ_ON_INT => {
                if self.current_command.get() != ProximityCommand::NoCommand {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.current_command
                    .set(ProximityCommand::ReadProximityOnInterrupt);
                self.thresholds.set(Thresholds {
                    lower: argument0 as u8,
                    upper: argument1 as u8,
                });
                if let Some(val) = self.upcall_on_command.get() {
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

const DRIVER_NUM: u32 = 0x60005;

// Command IDs

const EXISTS: u32 = 0;
const READ: u32 = 1;
const READ_ON_INT: u32 = 2;
