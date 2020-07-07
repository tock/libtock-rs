// This example just prints "Hello Tock World" to the terminal.
// Run `tockloader listen`, or use any serial program of your choice
//  (e.g. `screen`, `minicom`) to view the message.

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
