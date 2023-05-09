//! A simple libtock-rs example. Checks for adc driver
//! and samples the sensor every 2 seconds.

#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::console::Console;

use libtock::alarm::{Alarm, Milliseconds};
use libtock::runtime::{set_main, stack_size};
use libtock::adc::Adc;

set_main! {main}
stack_size! {0x200}

fn main() {
    match Adc::exists() {
        Ok(()) => writeln!(Console::writer(), "adc driver available").unwrap(),
        Err(_) => {
            writeln!(Console::writer(), "adc driver unavailable").unwrap();
            return;
        }
    }

    loop {
        match Adc::read_single_sample_sync() {
            Ok(adc_val) => writeln!(
                Console::writer(),
                "Sample: {}\n",
                adc_val
            )
            .unwrap(),
            Err(_) => writeln!(Console::writer(), "error while reading sample",).unwrap(),
        }

        Alarm::sleep_for(Milliseconds(2000)).unwrap();
    }
}