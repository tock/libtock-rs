#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::result::TockResult;
use libtock::temperature;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut console = Console::default();
    let temperature = temperature::measure_temperature().await?;
    writeln!(console, "Temperature: {}", temperature).map_err(Into::into)
}
