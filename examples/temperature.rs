#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::temperature;
use libtock_support_macros::libtock_main;

#[libtock_main]
async fn main() -> libtock::result::TockResult<()> {
    let mut console = Console::new();
    let temperature = temperature::measure_temperature().await?;
    writeln!(console, "Temperature: {}", temperature).map_err(Into::into)
}
