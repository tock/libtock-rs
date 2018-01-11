#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::button::ButtonState;
use tock::button::Buttons;
use tock::console::Console;
use tock::timer;

fn main() {
    let mut console = Console::new();
    let mut buttons = Buttons::without_callback().unwrap();
    let mut button = buttons.into_iter().next().unwrap();
    let button = button.enable().unwrap();

    loop {
        match button.read() {
            ButtonState::Pressed => console.write(String::from("pressed\n")),
            ButtonState::Released => console.write(String::from("released\n")),
        }
        timer::delay_ms(500);
    }
}
