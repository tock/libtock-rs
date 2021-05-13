#![no_std]

use libtock::alarm::Duration;
use libtock::result::TockResult;

libtock_core::stack_size! {0x800}

// Example works on P0.03
#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let mut gpio_driver = drivers.gpio.init_driver()?;
    let mut timer_driver = drivers.alarm.create_timer_driver();
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
