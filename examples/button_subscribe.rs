#![no_std]

use core::fmt::Write;
use futures::future;
use libtock::buttons;
use libtock::buttons::ButtonState;
use libtock::console::Console;
use libtock::result::TockResult;

// FIXME: Hangs up when buttons are pressed rapidly. Yielding in callback leads to stack overflow.
async fn main() -> TockResult<()> {
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
        .ok()
        .unwrap();
    });

    let mut buttons = with_callback.init()?;

    for mut button in &mut buttons {
        button.enable()?;
    }

    future::pending().await
}
