#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::button;
use tock::console::Console;
use tock::timer;

fn main() {
    let mut console = Console::new();
    let button = button::get(0).unwrap();
    let button = button.initialize(None).unwrap();

    loop {
        if button.read() {
            console.write(String::from("true\n"));
        } else {
            console.write(String::from("false\n"));
        }
        timer::delay_ms(500);
    }
}
