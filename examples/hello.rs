#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;
use libtock::Hardware;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware { console_driver } = libtock::retrieve_hardware()?;
    let mut console = console_driver.create_console();
    let context = timer::DriverContext::create()?;
    let mut driver = context.create_timer_driver()?;
    let timer_driver = driver.activate()?;

    for i in 0.. {
        writeln!(console, "Hello world! {}", i)?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }

    Ok(())
}
