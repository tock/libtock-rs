#![no_std]

use core::fmt::Write;
use futures::future;
use libtock::buttons::ButtonState;
use libtock::result::TockResult;
use libtock::Hardware;

// FIXME: Hangs up when buttons are pressed rapidly. Yielding in callback leads to stack overflow.
#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware {
        console_driver,
        button_driver,
        ..
    } = libtock::retrieve_hardware()?;
    let mut console = console_driver.create_console();
    let mut with_callback = button_driver.with_callback(|button_num: usize, state| {
        writeln!(
            console,
            "Button: {} - State: {}",
            button_num,
            match state {
                ButtonState::Pressed => "pressed",
                ButtonState::Released => "released",
            }
        )
        .ok()
        .unwrap();
    });

    let mut buttons = with_callback.init()?;

    for mut button in &mut buttons {
        button.enable()?;
    }

    future::pending().await
}
