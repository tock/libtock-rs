#![no_std]

use core::fmt::Write;
use libtock::adc;
use libtock::console::Console;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let context = timer::DriverContext::create()?;
    let mut driver = context.create_timer_driver().unwrap();
    let timer_driver = driver.activate()?;

    let mut console = Console::new();
    let mut with_callback = adc::with_callback(|channel: usize, value: usize| {
        writeln!(console, "channel: {}, value: {}", channel, value).unwrap();
    });

    let adc = with_callback.init()?;

    loop {
        adc.sample(0)?;
        timer_driver.sleep(Duration::from_ms(2000)).await?;
    }
}
