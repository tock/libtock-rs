#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::fmt::Write;
use alloc::string::String;
use tock::console::Console;

// TODO: Make alloc::string::ToString work
// TODO: Make write! work
fn main() {
    let mut console = Console::new();

    for _ in 0.. {
        console.write(String::from("Hello world!\n"));
        tock::timer::delay_ms(500);
    }
}
