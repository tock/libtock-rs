#![no_std]

use libtock::alarm::Duration;
use libtock::println;
use libtock::result::TockResult;

libtock_core::stack_size! {0x800}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let mut timer_driver = drivers.alarm.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    drivers.console.create_console();

    loop {
        println!("Mag:   {}\n", drivers.ninedof.read_magnetometer()?);
        println!("Accel: {}\n", drivers.ninedof.read_acceleration()?);
        println!("Gyro:  {}\n", drivers.ninedof.read_gyroscope()?);
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
