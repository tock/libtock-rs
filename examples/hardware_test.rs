#![feature(alloc)]
#![no_std]

extern crate alloc;

use alloc::string::String;
use tock::console::Console;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut console = Console::new();
    console.write(String::from("[test-results]\n"));
    let mut string = String::from("heap_test = \"Heap ");
    string.push_str("works.\"\n");
    console.write(string);

    for _ in 0.. {
        timer::sleep(Duration::from_ms(500))
    }
}
