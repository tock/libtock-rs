//! This example shows how to use the sound pressure driver.
//! It checks for the sound pressure driver and samples the sensor every second.

#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::console::Console;

use libtock::alarm::{Alarm, Milliseconds};
use libtock::runtime::{set_main, stack_size};
use libtock::sound_pressure::SoundPressure;

set_main! {main}
stack_size! {0x200}

fn main() {
    if SoundPressure::exists().is_err() {
        writeln!(Console::writer(), "Sound pressure driver not found").unwrap();
        return;
    }

    writeln!(Console::writer(), "Sound pressure driver found").unwrap();
    let enable = SoundPressure::enable();
    match enable {
        Ok(()) => {
            writeln!(Console::writer(), "Sound pressure driver enabled").unwrap();
            loop {
                match SoundPressure::read_sync() {
                    Ok(sound_pressure_val) => writeln!(
                        Console::writer(),
                        "Sound Pressure: {}\n",
                        sound_pressure_val
                    )
                    .unwrap(),
                    Err(_) => {
                        writeln!(Console::writer(), "error while reading sound pressure",).unwrap()
                    }
                }
                Alarm::sleep_for(Milliseconds(1000)).unwrap();
            }
        }
        Err(_e) => writeln!(Console::writer(), "Sound pressure driver enable failed",).unwrap(),
    }
}
