//! A simple libtock-rs example. Checks for adc driver
//! and samples the sensor every 2 seconds.

#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::console::Console;

use libtock::adc::Adc;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x200}

fn main() {
    if Adc::exists().is_err() {
        writeln!(Console::writer(), "adc driver unavailable").unwrap();
        return;
    }
    //getting the number of channels
    let number_channels: usize = match Adc::get_number_of_channels() {
        Ok(channel) => channel as usize,
        Err(_) => {
            writeln!(Console::writer(), "can't get number of channels").unwrap();
            return;
        }
    };
    loop {
        for i in 0..number_channels {
            match Adc::read_single_sample_sync(i) {
                Ok(adc_val) => writeln!(Console::writer(), "Sample: {}\n", adc_val).unwrap(),
                Err(_) => writeln!(Console::writer(), "error while reading sample",).unwrap(),
            }

            Alarm::sleep_for(Milliseconds(2000)).unwrap();
        }
    }
}
