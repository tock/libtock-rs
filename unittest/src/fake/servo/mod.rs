use std::cell::Cell;

use crate::DriverInfo;
use libtock_platform::{CommandReturn, ErrorCode};

pub struct Servo<const NUM_SERVO: usize> {
    servo: [Cell<u16>; NUM_SERVO],
}

impl<const NUM_SERVO: usize> Servo<NUM_SERVO> {
    pub fn new() -> std::rc::Rc<Servo<NUM_SERVO>> {
        #[allow(clippy::declare_interior_mutable_const)]
        const ANGLE: Cell<u16> = Cell::new(0);
        std::rc::Rc::new(Servo {
            servo: [ANGLE; NUM_SERVO],
        })
    }
}

impl<const NUM_SERVO: usize> crate::fake::SyscallDriver for Servo<NUM_SERVO> {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM)
    }

    fn command(&self, command_num: u32, servo_index: u32, angle: u32) -> CommandReturn {
        match command_num {
            EXISTS => crate::command_return::success(),
            SERVO_COUNT => crate::command_return::success_u32(NUM_SERVO as u32),
            SET_ANGLE => {
                if servo_index >= NUM_SERVO as u32 {
                    crate::command_return::failure(ErrorCode::NoDevice)
                } else if angle <= 180 {
                    self.servo[servo_index as usize].set(angle as u16);
                    crate::command_return::success()
                } else {
                    crate::command_return::failure(ErrorCode::Fail)
                }
            }
            // Return the current angle.
            GET_ANGLE => {
                if servo_index >= NUM_SERVO as u32 {
                    crate::command_return::failure(ErrorCode::NoDevice)
                } else {
                    let angle = self.servo[servo_index as usize].get();
                    crate::command_return::success_u32(angle as u32)
                }
            }
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Implementation details below
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x90009;

// Command numbers
const EXISTS: u32 = 0;
const SERVO_COUNT: u32 = 1;
const SET_ANGLE: u32 = 2;
const GET_ANGLE: u32 = 3;
