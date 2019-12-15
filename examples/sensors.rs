#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::sensors::*;
use libtock::timer;
use libtock::timer::Duration;
use libtock_support_macros::libtock_main;

#[libtock_main]
async fn main() -> libtock::result::TockResult<()> {
    let mut console = Console::new();
    let mut humidity = HumiditySensor;
    let mut temperature = TemperatureSensor;
    let mut light = AmbientLightSensor;
    let mut ninedof = Ninedof::new();
    loop {
        writeln!(console, "Humidity:    {}\n", humidity.read()?)?;
        writeln!(console, "Temperature: {}\n", temperature.read()?)?;
        writeln!(console, "Light:       {}\n", light.read()?)?;
        writeln!(console, "Accel:       {}\n", ninedof.read_acceleration()?)?;
        timer::sleep(Duration::from_ms(500)).await?;
    }
}
