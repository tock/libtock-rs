#![no_std]

extern crate tock;

use core::fmt::Write;
use tock::buttons;
use tock::buttons::ButtonState;
use tock::console::Console;
use tock::timer;
use tock::timer::Duration;

// FIXME: Hangs up when buttons are pressed rapidly - problem in console?
fn main() {
    let mut console = Console::new();

    let mut with_callback = buttons::with_callback(|button_num: usize, state| {
        let state_as_text = match state {
            ButtonState::Pressed => "pressed",
            ButtonState::Released => "released",
        };
        writeln!(console, "Button: {} - State: {}", button_num, state_as_text);
    });

    let mut buttons = with_callback.init().unwrap();

    for mut button in &mut buttons {
        button.enable().unwrap();
    }

    loop {
        timer::sleep(Duration::from_ms(500));
    }
}
