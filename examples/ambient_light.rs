//! A simple libtock-rs example. Checks for ambient light driver
//! and samples the sensor every 2 seconds.

#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::console::Console;

use libtock::alarm::{Alarm, Milliseconds};
use libtock::ambient_light::AmbientLight;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x200}

fn main() {
    if AmbientLight::exists().is_err() {
        writeln!(Console::writer(), "ambient light driver unavailable").unwrap();
        return;
    }

    loop {
        match AmbientLight::read_intensity_sync() {
            Ok(intensity_val) => writeln!(
                Console::writer(),
                "Light intensity: {} lux\n",
                intensity_val
            )
            .unwrap(),
            Err(_) => writeln!(Console::writer(), "error while reading light intensity",).unwrap(),
        }

        Alarm::sleep_for(Milliseconds(2000)).unwrap();
    }
}
