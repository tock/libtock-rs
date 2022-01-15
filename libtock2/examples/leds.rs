//! An extremely simple libtock-rs example. Just prints out a few numbers using
//! the LowLevelDebug capsule then terminates.

#![no_main]
#![no_std]

use libtock2::leds::Leds;
use libtock2::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x100}

fn main() {
    Leds::on(0);
}
