//! An extremely simple libtock-rs example. Just prints out a few numbers using
//! the LowLevelDebug capsule then terminates.

#![no_main]
#![no_std]

use libtock::low_level_debug::LowLevelDebug;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x100}

fn main() {
    LowLevelDebug::print_1(1);
    LowLevelDebug::print_2(2, 3);
}
