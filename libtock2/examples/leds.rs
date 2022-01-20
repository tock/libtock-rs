//! An extremely simple libtock-rs example. Just turns on all the LEDs.

#![no_main]
#![no_std]

use libtock2::leds::Leds;
use libtock2::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x100}

fn main() {
    if let Some(leds_count) = Leds::count() {
        for led_index in 0..leds_count {
            Leds::on(led_index as u32);
        }
    }
}
