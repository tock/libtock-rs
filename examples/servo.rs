#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size};
use libtock::servo::Servo;
use libtock_platform::ErrorCode;

set_main! {main}
stack_size! {0x300}

fn main() {
    //Checks if the driver exists.
    if Err(ErrorCode::Fail) == Servo::exists() {
        writeln!(Console::writer(), "The driver could not be found").unwrap();
        return;
    }
    let servo_count = Servo::servo_count().unwrap();

    writeln!(
        Console::writer(),
        "The number of available servomotors is {:?}",
        servo_count
    )
    .unwrap();

    let index: u32 = 0; // the first index available.

    // Changes the angle of the servomotor from 0 to 180 degrees (waiting 0.1 ms between every change).
    // "i" represents the angle we set the servomotor at.
    for i in 0..180 {
        let val1 = Servo::set_angle(index, i); // stores the value returned by set_angle
        let val2 = Servo::get_angle(index); // stores the value returned by get_angle

        if val1 == Err(ErrorCode::Fail) {
            writeln!(
                Console::writer(),
                "The provided angle exceeds the servo's limit"
            )
            .unwrap();
        } else if val2 == Err(ErrorCode::NoSupport) {
            writeln!(Console::writer(), "The servo cannot return its angle").unwrap();
        } else if val1 == Err(ErrorCode::NoDevice) {
            writeln!(
                Console::writer(),
                "The index exceeds the number of provided servomotors"
            )
            .unwrap();
        } else if val2 == Err(ErrorCode::NoDevice) {
            writeln!(
                Console::writer(),
                "The index exceeds the number of provided servomotors"
            )
            .unwrap();
        }
        Alarm::sleep_for(Milliseconds(100)).unwrap();
    }
}
