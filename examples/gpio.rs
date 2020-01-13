#![no_std]

use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Drivers;

// Example works on P0.03
#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        mut gpio_driver_factory,
        mut timer_context,
        ..
    } = libtock::retrieve_drivers()?;

    let mut gpio_driver = gpio_driver_factory.init_driver()?;
    let mut timer_driver = timer_context.create_timer_driver();
    let timer_driver = timer_driver.activate()?;

    let mut gpio = gpio_driver.gpios().next().unwrap();
    let gpio_out = gpio.enable_output()?;
    loop {
        gpio_out.set_high()?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
        gpio_out.set_low()?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
