#![no_std]

use libtock::alarm::Duration;
use libtock::println;
use libtock::result::TockResult;
use libtock::sensors::Sensor;

libtock_core::stack_size! {0x800}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let mut timer_driver = drivers.alarm.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    drivers.console.create_console();

    loop {
        println!("Humidity:    {}\n", drivers.humidity_sensor.read()?);
        println!("Temperature: {}\n", drivers.temperature_sensor.read()?);
        println!("Light:       {}\n", drivers.ambient_light_sensor.read()?);
        println!("Accel:       {}\n", drivers.ninedof.read_acceleration()?);
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
