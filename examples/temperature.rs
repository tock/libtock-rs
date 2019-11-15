#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::result::TockResult;
use libtock::temperature;

async fn main() -> TockResult<()> {
    let mut console = Console::new();
    let temperature = temperature::measure_temperature().await?;
    writeln!(console, "Temperature: {}", temperature).map_err(Into::into)
}
