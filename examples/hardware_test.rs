#![feature(alloc)]
#![no_std]

extern crate alloc;

use alloc::string::String;
use core::fmt::Write;
use tock::console::Console;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut console = Console::new();
    writeln!(console, "[test-results]").unwrap();
    let mut string = String::from("heap_test = \"Heap ");
    string.push_str("works.\"\n");
    writeln!(console, "{}", string).unwrap();

    for _ in 0.. {
        timer::sleep(Duration::from_ms(500))
    }
}
