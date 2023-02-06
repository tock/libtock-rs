//! A simple libtock-rs example. Checks for temperature driver
//! and samples the sensor every 2 seconds.

#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::console::Console;

use libtock::alarm::{Alarm, Milliseconds};
use libtock::runtime::{set_main, stack_size};
use libtock::temperature::Temperature;

set_main! {main}
stack_size! {0x200}

fn main() {
    match Temperature::exists() {
        Ok(()) => writeln!(Console::writer(), "temperature driver available").unwrap(),
        Err(_) => {
            writeln!(Console::writer(), "temperature driver unavailable").unwrap();
            return;
        }
    }

    loop {
        match Temperature::read_temperature_sync() {
            Ok(temp_val) => writeln!(
                Console::writer(),
                "Temperature: {}{}.{}*C\n",
                if temp_val > 0 { "" } else { "-" },
                i32::abs(temp_val) / 100,
                i32::abs(temp_val) % 100
            )
            .unwrap(),
            Err(_) => writeln!(Console::writer(), "error while reading temperature",).unwrap(),
        }

        Alarm::sleep_for(Milliseconds(2000)).unwrap();
    }
}
