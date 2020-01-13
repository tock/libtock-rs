#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Drivers;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        mut buttons_driver_factory,
        mut timer_context,
        console_driver,
        ..
    } = libtock::retrieve_drivers()?;

    let buttons_driver = buttons_driver_factory.init_driver()?;
    let mut timer_driver = timer_context.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    let mut console = console_driver.create_console();

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
