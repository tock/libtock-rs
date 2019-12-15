#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::timer;
use libtock::timer::Duration;
use libtock_support_macros::libtock_main;

#[libtock_main]
async fn main() -> libtock::result::TockResult<()> {
    let mut console = Console::new();

    for i in 0.. {
        writeln!(console, "Hello world! {}", i)?;
        timer::sleep(Duration::from_ms(500)).await?;
    }

    Ok(())
}
