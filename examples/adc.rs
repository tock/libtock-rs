#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Drivers;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        mut adc_driver_factory,
        mut timer_context,
        console_driver,
        ..
    } = libtock::retrieve_drivers()?;

    let adc_driver = adc_driver_factory.init_driver()?;
    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;
    let mut console = console_driver.create_console();

    let mut callback = |channel, value| {
        writeln!(console, "channel: {}, value: {}", channel, value).unwrap();
    };

    let _subscription = adc_driver.subscribe(&mut callback)?;

    loop {
        adc_driver.sample(0)?;
        timer_driver.sleep(Duration::from_ms(2000)).await?;
    }
}
