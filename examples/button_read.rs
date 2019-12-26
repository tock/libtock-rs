#![no_std]

use core::fmt::Write;
use libtock::buttons;
use libtock::buttons::ButtonState;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Hardware;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware {
        console_driver,
        timer_context,
        ..
    } = libtock::retrieve_hardware()?;
    let mut console = console_driver.create_console();
    let mut with_callback = buttons::with_callback(|_, _| {});
    let mut buttons = with_callback.init()?;
    let mut button = buttons.iter_mut().next().unwrap();
    let button = button.enable()?;

    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;

    loop {
        match button.read()? {
            ButtonState::Pressed => writeln!(console, "pressed"),
            ButtonState::Released => writeln!(console, "released"),
        }?;

        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
