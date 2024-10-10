#![no_std]

use libtock_platform::{ErrorCode, Syscalls};

pub struct Servo<S: Syscalls>(S);

impl<S: Syscalls> Servo<S> {
    /// Check whether the driver exists.
    pub fn exists() -> Result<(), ErrorCode> {
        let val = S::command(DRIVER_NUM, EXISTS, 0, 0).is_success();
        if val {
            Ok(())
        } else {
            Err(ErrorCode::Fail)
        }
    }
    /// Returns the number of the servomotors available.
    pub fn servo_count() -> Result<u32, ErrorCode> {
        S::command(DRIVER_NUM, SERVO_COUNT, 0, 0).to_result()
    }

    /// Changes the angle of the servo.
    /// Return values:
    ///
    /// - `Ok(())`: The attempt at changing the angle was successful.
    /// - `FAIL`: Cannot change the angle.
    /// - `INVAL`: The value exceeds u16, indicating it's incorrect
    ///   since servomotors can only have a maximum of 360 degrees.
    /// - `NODEVICE`: The index exceeds the number of servomotors provided.
    ///  # Arguments
    /// - `angle` - the variable that receives the angle
    ///   (in degrees from 0 to 180) from the servo driver.
    /// - `index` - the variable that receives the index of the servomotor.
    pub fn set_angle(index: u32, angle: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, SET_ANGLE, index, angle).to_result()
    }

    /// Returns the angle of the servo.
    /// Return values:
    ///
    /// - `angle`: The value, in angles from 0 to 360, of the servo.
    /// - `NOSUPPORT`:  The servo cannot return its angle.
    /// - `NODEVICE`: The index exceeds the number of servomotors provided.
    ///  # Arguments
    /// - `index` - the variable that receives the index of the servomotor.
    pub fn get_angle(index: u32) -> Result<u32, ErrorCode> {
        S::command(DRIVER_NUM, GET_ANGLE, index, 0).to_result()
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x90009;

// Command IDs
const EXISTS: u32 = 0;
const SERVO_COUNT: u32 = 1;
const SET_ANGLE: u32 = 2;
const GET_ANGLE: u32 = 3;
