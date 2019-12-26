#![no_std]

use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Hardware;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware {
        mut led_driver,
        timer_context,
        ..
    } = libtock::retrieve_hardware()?;

    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;

    // Blink the LEDs in a binary count pattern and scale
    // to the number of LEDs on the board.
    let mut count: usize = 0;
    loop {
        for mut led in led_driver.all() {
            let i = led.number();
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
