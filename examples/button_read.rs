#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let buttons_driver = drivers.buttons.init_driver()?;
    let mut timer_driver = drivers.timer.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    let mut console = drivers.console.create_console();

    loop {
        for button in buttons_driver.buttons() {
            writeln!(
                console,
                "button {}: {:?}",
                button.button_num(),
                button.read()?
            )?;
        }
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
