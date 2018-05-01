#![no_std]

extern crate tock;

use tock::buttons;
use tock::buttons::ButtonState;
use tock::led;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut with_callback = buttons::with_callback(|button_num, state| {
        let i = button_num;
        match state {
            ButtonState::Pressed => led::get(i).unwrap().toggle(),
            ButtonState::Released => (),
        };
    });

    let mut buttons = with_callback.init().unwrap();

    for mut button in &mut buttons {
        button.enable().unwrap();
    }

    loop {
        timer::sleep(Duration::from_ms(500));
    }
}
