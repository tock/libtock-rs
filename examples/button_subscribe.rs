#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::button::ButtonState;
use tock::button::Buttons;
use tock::console::Console;
use tock::fmt;
use tock::timer;

// FIXME: Hangs up when buttons are pressed rapidly - problem in console?
fn main() {
    let mut console = Console::new();

    let mut buttons = Buttons::with_callback(|button_num: usize, state| {
        console.write(String::from("\nButton: "));
        console.write(fmt::u32_as_hex(button_num as u32));
        console.write(String::from(" - State: "));
        console.write(String::from(match state {
            ButtonState::Pressed => "pressed",
            ButtonState::Released => "released",
        }));
    }).unwrap();

    for mut button in &mut buttons {
        button.enable().unwrap();
    }

    loop {
        timer::delay_ms(500);
    }
}
