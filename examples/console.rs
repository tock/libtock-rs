//! An extremely simple libtock-rs example. Just prints out a message
//! using the Console capsule, then terminates.

#![no_main]
#![no_std]
//use core::fmt::Write;

use ufmt::{uWrite, uwriteln};
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x100}

fn main() {
    let mut writer = Console::writer();
    uwriteln!(writer, "Hello1").unwrap();
    uwriteln!(writer, "Hello2").unwrap();
    uwriteln!(writer, "Hello3").unwrap();
    uwriteln!(writer, "Hello4").unwrap();
    uwriteln!(writer, "Hello5").unwrap();
    uwriteln!(writer, "Hello6").unwrap();
    uwriteln!(writer, "Hello7").unwrap();
}
