#![no_std]

use libtock::result::TockResult;
use libtock::timer::Duration;

libtock_core::stack_size! {0x400}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let leds_driver = drivers.leds.init_driver()?;
    let mut timer_driver = drivers.timer.create_timer_driver();
    let timer_driver = timer_driver.activate()?;

    // Blink the LEDs in a binary count pattern and scale
    // to the number of LEDs on the board.
    let mut count: usize = 0;
    loop {
        for led in leds_driver.leds() {
            let i = led.led_num();
            if count & (1 << i) == (1 << i) {
                led.on()?;
            } else {
                led.off()?;
            }
        }
        count = count.wrapping_add(1);

        // This delay uses an underlying timer in the kernel.
        timer_driver.sleep(Duration::from_ms(250)).await?;
    }
}
