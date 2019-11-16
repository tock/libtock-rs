#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut console = Console::new();
    let context = timer::DriverContext::create()?;
    let mut driver = context.create_timer_driver().unwrap();
    let timer_driver = driver.activate()?;

    for i in 0.. {
        writeln!(console, "Hello world! {}", i)?;
        timer_driver.parallel_sleep(Duration::from_ms(500)).await?;
    }

    Ok(())
}
