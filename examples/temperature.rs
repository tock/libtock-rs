#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Drivers;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        mut temperature_driver_factory,
        mut timer_context,
        console_driver,
        ..
    } = libtock::retrieve_drivers()?;

    let mut temperature_driver = temperature_driver_factory.init_driver()?;
    let mut timer_driver = timer_context.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    let mut console = console_driver.create_console();

    loop {
        let temperature = temperature_driver.measure_temperature().await?;
        writeln!(console, "Temperature: {}", temperature)?;
        timer_driver.sleep(Duration::from_ms(1000)).await?;
    }
}
