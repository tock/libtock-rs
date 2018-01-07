#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::console::Console;
use tock::gpio::{GpioPinUnitialized, InputMode};
use tock::timer;

// example works on p0.03
fn main() {
    let mut console = Console::new();
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_read(None, InputMode::PullDown).unwrap();

    loop {
        if pin.read() {
            console.write(String::from("true\n"));
        } else {
            console.write(String::from("false\n"));
        }
        timer::delay_ms(500);
    }
}
