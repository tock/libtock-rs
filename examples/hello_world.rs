// This example just prints "Hello Tock World" to the terminal.
// Run `tockloader listen`, or use any serial program of your choice
//  (e.g. `screen`, `minicom`) to view the message.

#![no_std]

use libtock::println;
use libtock::result::TockResult;

#[libtock::main]
async fn main() -> TockResult<()> {
    let drivers = libtock::retrieve_drivers()?;

    drivers.console.create_console();

    println!("Hello Tock World");

    Ok(())
}
