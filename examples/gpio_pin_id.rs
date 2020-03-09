#![no_std]

use libtock::result::TockResult;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let mut gpio_driver = drivers.gpio.init_driver()?;
    let mut timer_driver = drivers.timer.create_timer_driver();
    let timer_driver = timer_driver.activate()?;

    // search for pin PE09 (nucleo f429zi)
    // msb 4 bits, port E (4th port, staring from 0) - 0100
    // lsb 4 bits, pin 9 - 1001
    if let Some(mut gpio) = gpio_driver.gpios().next_at(0b0100_1001) {
        let gpio_out = gpio.enable_output()?;
        loop {
            gpio_out.set_high()?;
            timer_driver.sleep(Duration::from_ms(1000)).await?;
            gpio_out.set_low()?;
            timer_driver.sleep(Duration::from_ms(1000)).await?;
        }
    }

    panic!("The requested GPIO is not available");
}
