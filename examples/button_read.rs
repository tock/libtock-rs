#![no_std]

use core::fmt::Write;
use libtock::buttons;
use libtock::buttons::ButtonState;
use libtock::console::Console;
use libtock::timer;
use libtock::timer::Duration;

fn main() {
    let mut console = Console::new();
    let mut with_callback = buttons::with_callback(|_, _| {});
    let mut buttons = with_callback.init().unwrap();
    let mut button = buttons.iter_mut().next().unwrap();
    let button = button.enable().unwrap();

    loop {
        match button.read() {
            ButtonState::Pressed => writeln!(console, "pressed"),
            ButtonState::Released => writeln!(console, "released"),
        }
        .unwrap();
        timer::sleep(Duration::from_ms(500));
    }
}
