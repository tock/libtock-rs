#![no_std]

use futures::future;
use libtock::buttons;
use libtock::buttons::ButtonState;
use libtock::led;
use libtock::result::TockResult;

async fn main() -> TockResult<()> {
    let mut with_callback = buttons::with_callback(|button_num: usize, state| {
        match state {
            ButtonState::Pressed => led::get(button_num).unwrap().toggle().ok().unwrap(),
            ButtonState::Released => (),
        };
    });

    let mut buttons = with_callback.init()?;

    for mut button in &mut buttons {
        button.enable()?;
    }

    future::pending().await
}
