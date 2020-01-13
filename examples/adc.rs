#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let adc_driver = drivers.adc.init_driver()?;
    let mut timer_driver = drivers.timer.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    let mut console = drivers.console.create_console();

    let mut callback = |channel, value| {
        writeln!(console, "channel: {}, value: {}", channel, value).unwrap();
    };

    let _subscription = adc_driver.subscribe(&mut callback)?;

    loop {
        adc_driver.sample(0)?;
        timer_driver.sleep(Duration::from_ms(2000)).await?;
    }
}
