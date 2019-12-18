#![no_std]
/**
 * This example sends many messages to the console driver.
 **/
use core::fmt::Write;
use libtock::console::Console;
use libtock::result::TockResult;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut console = Console::new();

    for i in 0.. {
        for j in 0..i {
            writeln!(console, "Hello world! {}", j)?;
        }
        let x: [u8; 0x100] = [i; 0x100];
        writeln!(console, "x = {:?}", &x as &[u8])?;
    }

    Ok(())
}
