#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::Drivers;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        console_driver,
        mut temperature_driver,
        ..
    } = libtock::retrieve_drivers()?;
    let mut console = console_driver.create_console();
    let temperature = temperature_driver.measure_temperature().await?;
    writeln!(console, "Temperature: {}", temperature).map_err(Into::into)
}
