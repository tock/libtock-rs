#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::temperature;
use libtock::Hardware;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware { console_driver, .. } = libtock::retrieve_hardware()?;
    let mut console = console_driver.create_console();
    let temperature = temperature::measure_temperature().await?;
    writeln!(console, "Temperature: {}", temperature).map_err(Into::into)
}
