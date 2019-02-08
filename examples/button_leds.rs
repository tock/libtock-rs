#![no_std]

use libtock::buttons;
use libtock::buttons::ButtonState;
use libtock::led;
use libtock::timer;
use libtock::timer::Duration;

fn main() {
    let mut with_callback = buttons::with_callback(|button_num: usize, state| {
        let i = button_num as isize;
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
