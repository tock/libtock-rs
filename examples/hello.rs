#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Drivers;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        console_driver,
        timer_context,
        ..
    } = libtock::retrieve_drivers()?;
    let mut console = console_driver.create_console();
    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;

    for i in 0.. {
        writeln!(console, "Hello world! {}", i)?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }

    Ok(())
}
