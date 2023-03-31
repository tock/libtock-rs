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
    writeln!(Console::writer(), "Sound Pressure Example\n").unwrap();
    match SoundPressure::exists() {
        Ok(()) => writeln!(Console::writer(), "sound pressure driver available").unwrap(),
        Err(_) => {
            writeln!(Console::writer(), "sound pressure driver unavailable").unwrap();
            return;
        }
    }

    let _ = SoundPressure::sound_pressure_enabled();
    writeln!(Console::writer(), "Sound Pressure Enabled:\n",).unwrap();

    loop {
        match SoundPressure::read_pressure_sync() {
            Ok(sound_pressure_val) => writeln!(
                Console::writer(),
                "Sound Pressure: {}\n",
                sound_pressure_val
            )
            .unwrap(),
            Err(_) => writeln!(Console::writer(), "error while reading sound pressure",).unwrap(),
        }
        Alarm::sleep_for(Milliseconds(1000)).unwrap();
    }
}
