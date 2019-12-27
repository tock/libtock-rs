#![no_std]

use core::fmt::Write;
use libtock::result::TockResult;
use libtock::sensors::*;
use libtock::timer::Duration;
use libtock::Hardware;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware {
        console_driver,
        timer_context,
        mut temperature_sensor,
        mut humidity_sensor,
        mut ambient_light_sensor,
        mut ninedof_driver,
        ..
    } = libtock::retrieve_hardware()?;
    let mut console = console_driver.create_console();
    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;

    loop {
        writeln!(console, "Humidity:    {}\n", humidity_sensor.read()?)?;
        writeln!(console, "Temperature: {}\n", temperature_sensor.read()?)?;
        writeln!(console, "Light:       {}\n", ambient_light_sensor.read()?)?;
        writeln!(
            console,
            "Accel:       {}\n",
            ninedof_driver.read_acceleration()?
        )?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
