#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Hardware;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware {
        console_driver,
        timer_context,
        adc_driver,
        ..
    } = libtock::retrieve_hardware()?;

    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;
    let mut console = console_driver.create_console();
    let mut with_callback = adc_driver.with_callback(|channel: usize, value: usize| {
        writeln!(console, "channel: {}, value: {}", channel, value).unwrap();
    });

    let adc = with_callback.init()?;

    loop {
        adc.sample(0)?;
        timer_driver.sleep(Duration::from_ms(2000)).await?;
    }
}
