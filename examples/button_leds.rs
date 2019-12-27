#![no_std]

use futures::future;
use libtock::buttons::ButtonState;
use libtock::result::TockResult;
use libtock::Hardware;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware {
        mut led_driver,
        button_driver,
        ..
    } = libtock::retrieve_hardware()?;

    let mut with_callback = button_driver.with_callback(|button_num: usize, state| {
        match state {
            ButtonState::Pressed => led_driver.get(button_num).unwrap().toggle().ok().unwrap(),
            ButtonState::Released => (),
        };
    });

    let mut buttons = with_callback.init()?;

    for mut button in &mut buttons {
        button.enable()?;
    }

    future::pending().await
}
