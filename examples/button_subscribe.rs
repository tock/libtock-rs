#![no_std]

use core::fmt::Write;
use libtock::buttons;
use libtock::buttons::ButtonState;
use libtock::console::Console;
use libtock::timer;
use libtock::timer::Duration;

// FIXME: Hangs up when buttons are pressed rapidly - problem in console?
fn main() {
    let mut console = Console::new();

    let mut with_callback = buttons::with_callback(|button_num: usize, state| {
        writeln!(
            console,
            "Button: {} - State: {}",
            button_num,
            match state {
                ButtonState::Pressed => "pressed",
                ButtonState::Released => "released",
            }
        )
        .unwrap();
    });

    let mut buttons = with_callback.init().unwrap();

    for mut button in &mut buttons {
        button.enable().unwrap();
    }

    loop {
        timer::sleep(Duration::from_ms(500));
    }
}
