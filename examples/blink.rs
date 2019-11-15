#![no_std]

use libtock::led;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;

async fn main() -> TockResult<()> {
    let num_leds = led::count()?;

    // Blink the LEDs in a binary count pattern and scale
    // to the number of LEDs on the board.
    let mut count: usize = 0;
    loop {
        for i in 0..num_leds {
            if count & (1 << i) == (1 << i) {
                led::get(i).unwrap().on()?;
            } else {
                led::get(i).unwrap().off()?;
            }
        }
        count = count.wrapping_add(1);

        // This delay uses an underlying timer in the kernel.
        timer::sleep(Duration::from_ms(250)).await?;
    }
}
