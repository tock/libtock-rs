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
    match NineDof::exists() {
        Ok(()) => writeln!(Console::writer(), "ninedof driver available").unwrap(),
        Err(_) => {
            writeln!(Console::writer(), "ninedof driver unavailable").unwrap();
            return;
        }
    }

    let mut x = 0;
    let mut y = 0;
    let mut z = 0;

    loop {
        match NineDof::read_accelerometer_sync(&mut x, &mut y, &mut z) {
            Ok(_data) => writeln!(
                Console::writer(),
                "Accelerometer: x: {}, y: {}, z: {}",
                x,
                y,
                z
            )
            .unwrap(),
            Err(_) => writeln!(Console::writer(), "error while reading accelerometer").unwrap(),
        }

        match NineDof::read_gyro_sync(&mut x, &mut y, &mut z) {
            Ok(_data) => {
                writeln!(Console::writer(), "Gyroscope: x: {}, y: {}, z: {}", x, y, z).unwrap()
            }
            Err(_) => writeln!(Console::writer(), "error while reading gyroscope").unwrap(),
        }

        match NineDof::read_magnetometer_sync(&mut x, &mut y, &mut z) {
            Ok(_data) => writeln!(
                Console::writer(),
                "Magnetometer: x: {}, y: {}, z: {}",
                x,
                y,
                z
            )
            .unwrap(),
            Err(_) => writeln!(Console::writer(), "error while reading magnetometer").unwrap(),
        }

        let acc_mag = NineDof::ninedof_read_accel_mag();
        writeln!(Console::writer(), "Magnitude of acceleration: {}", acc_mag).unwrap();

        Alarm::sleep_for(Milliseconds(700)).unwrap();
    }
}
