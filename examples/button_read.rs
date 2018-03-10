#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::buttons;
use tock::buttons::ButtonState;
use tock::console::Console;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut console = Console::new();
    let mut buttons = buttons::with_callback(|_, _| {}).unwrap();
    let mut button = buttons.iter_mut().next().unwrap();
    let button = button.enable().unwrap();

    loop {
        match button.read() {
            ButtonState::Pressed => console.write(String::from("pressed\n")),
            ButtonState::Released => console.write(String::from("released\n")),
        }
        timer::sleep(Duration::from_ms(500));
    }
}
