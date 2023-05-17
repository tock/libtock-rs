//! Libtock-rs example for the ninedof sensor.

#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::console::Console;

use libtock::alarm::{Alarm, Milliseconds};
use libtock::ninedof::NineDof;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x2000}

fn main() {
    if NineDof::exists().is_err() {
        writeln!(Console::writer(), "NineDof driver unavailable").unwrap();
        return;
    }

    writeln!(Console::writer(), "NineDof driver available").unwrap();
    loop {
        let accelerometer_data = NineDof::read_accelerometer_sync();
        let magnetomer_data = NineDof::read_magnetometer_sync();
        let gyroscope_data = NineDof::read_gyroscope_sync();

        match accelerometer_data {
            Ok(data) => {
                writeln!(
                    Console::writer(),
                    "Accelerometer: x: {}, y: {}, z: {}",
                    data.x,
                    data.y,
                    data.z
                )
                .unwrap();
            }
            Err(_) => writeln!(Console::writer(), "error while reading accelerometer").unwrap(),
        }

        match magnetomer_data {
            Ok(data) => {
                writeln!(
                    Console::writer(),
                    "Magnetometer: x: {}, y: {}, z: {}",
                    data.x,
                    data.y,
                    data.z
                )
                .unwrap();
            }
            Err(_) => writeln!(Console::writer(), "error while reading magnetometer").unwrap(),
        }

        match gyroscope_data {
            Ok(data) => {
                writeln!(
                    Console::writer(),
                    "Gyroscope: x: {}, y: {}, z: {}",
                    data.x,
                    data.y,
                    data.z
                )
                .unwrap();
            }
            Err(_) => writeln!(Console::writer(), "error while reading gyroscope").unwrap(),
        }
        Alarm::sleep_for(Milliseconds(700)).unwrap();
    }
}
