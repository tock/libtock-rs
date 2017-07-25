#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::String;
use tock::console::Console;

fn main() {
    let mut console = Console::new();
    console.write(String::from("Hello\n"));
}

