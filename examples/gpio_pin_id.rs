#![no_std]

use libtock::result::TockResult;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let gpio_driver = drivers.gpio.init_driver()?;
    let mut timer_driver = drivers.timer.create_timer_driver();
    let timer_driver = timer_driver.activate()?;

    // search for pin PE09 (nucleo f429zi)
    // msb 4 bits, port E (4th port, staring from 0) - 0100
    // lsb 4 bits, pin 9 - 1001
    if let Some(gpio) = gpio.gpio_by_id(0b01001001) {
        gpio.set_output();
        loop {
            gpio.set();
            timer_driver.sleep(Duration::from_ms(1000)).await?;
            gpio.clear();
            timer_driver.sleep(Duration::from_ms(1000)).await?;
        }
    }

    loop {}
}
