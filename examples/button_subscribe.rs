#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::buttons;
use tock::buttons::ButtonState;
use tock::buttons::ButtonsCallback;
use tock::console::Console;
use tock::fmt;
use tock::timer;
use tock::timer::Duration;

// FIXME: Hangs up when buttons are pressed rapidly - problem in console?
fn main() {
    let mut console = Console::new();

    let mut callback = ButtonsCallback::new(|button_num: usize, state| {
        console.write(String::from("\nButton: "));
        console.write(fmt::u32_as_hex(button_num as u32));
        console.write(String::from(" - State: "));
        console.write(String::from(match state {
            ButtonState::Pressed => "pressed",
            ButtonState::Released => "released",
        }));
    });

    let mut buttons = buttons::with_callback(&mut callback).unwrap();

    for mut button in &mut buttons {
        button.enable().unwrap();
    }

    loop {
        timer::sleep(Duration::from_ms(500));
    }
}
