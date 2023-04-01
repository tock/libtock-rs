//! Fake implementation of the NineDof API, documented here:
//!
//! Like the real API, `NineDof` controls a fake 9dof sensor. It provides
//! a function `set_value` used to immediately call an upcall with a 9dof value read by the sensor
//! and a function 'set_value_sync' used to call the upcall when the read command is received.

use crate::{DriverInfo, DriverShareRef};
use libtock_platform::{CommandReturn, ErrorCode};
use std::cell::Cell;

pub struct NineDof {
    busy: Cell<bool>,
    upcall_on_command: Cell<Option<NineDofData>>,
    share_ref: DriverShareRef,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NineDofData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl NineDof {
    pub fn new() -> std::rc::Rc<NineDof> {
        std::rc::Rc::new(NineDof {
            busy: Cell::new(false),
            upcall_on_command: Cell::new(None),
            share_ref: Default::default(),
        })
    }

    pub fn is_busy(&self) -> bool {
        self.busy.get()
    }
    pub fn set_value(&self, value: NineDofData) {
        if self.busy.get() {
            self.share_ref
                .schedule_upcall(0, (value.x as u32, value.y as u32, value.z as u32))
                .expect("Unable to schedule upcall");
            self.busy.set(false);
        }
    }
    pub fn set_value_sync(&self, value: NineDofData) {
        self.upcall_on_command.set(Some(value));
    }
}

impl crate::fake::SyscallDriver for NineDof {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_id: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_id {
            EXISTS => crate::command_return::success(),

            READ_ACCELEROMETER => {
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.busy.set(true);
                if let Some(val) = self.upcall_on_command.take() {
                    self.set_value(val);
                }
                crate::command_return::success()
            }
            READ_MAGNETOMETER => {
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.busy.set(true);
                if let Some(val) = self.upcall_on_command.take() {
                    self.set_value(val);
                }
                crate::command_return::success()
            }
            READ_GYRO => {
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

const DRIVER_NUM: u32 = 0x60004;

// Command IDs
const EXISTS: u32 = 0;
const READ_ACCELEROMETER: u32 = 1;
const READ_MAGNETOMETER: u32 = 100;
const READ_GYRO: u32 = 200;
