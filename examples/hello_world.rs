// This example just prints "Hello Tock World" to the terminal.

#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;

#[libtock::main]
async fn main() -> TockResult<()> {
    let drivers = libtock::retrieve_drivers()?;

    let mut console = drivers.console.create_console();

    writeln!(console, "Hello Tock World")?;

    Ok(())
}
