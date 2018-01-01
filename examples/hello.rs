#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::console::Console;

// TODO: Make alloc::string::ToString work
// TODO: Make write!/format! work
fn main() {
    let mut console = Console::new();

    for i in 0.. {
        console.write(String::from("Hello world! "));
        console.write(tock::fmt::u32_as_decimal(i));
        console.write(String::from("\n"));
        tock::timer::delay_ms(500);
    }
}
