#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let mut temperature_driver = drivers.temperature.init_driver()?;
    let mut timer_driver = drivers.timer.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    let mut console = drivers.console.create_console();

    loop {
        let temperature = temperature_driver.measure_temperature().await?;
        writeln!(console, "Temperature: {}", temperature)?;
        timer_driver.sleep(Duration::from_ms(1000)).await?;
    }
}
