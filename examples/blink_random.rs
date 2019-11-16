#![no_std]

use libtock::led;
use libtock::result::TockResult;
use libtock::rng;
use libtock::timer;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let context = timer::DriverContext::create()?;
    let mut driver = context.create_timer_driver().unwrap();
    let timer_driver = driver.activate()?;

    let num_leds = led::count()?;
    // blink_nibble assumes 4 leds.
    assert_eq!(num_leds, 4);

    let mut buf = [0; 64];
    loop {
        rng::fill_buffer(&mut buf).await?;

        for &x in buf.iter() {
            blink_nibble(x)?;
            timer_driver.parallel_sleep(Duration::from_ms(100)).await?;
            blink_nibble(x >> 4)?;
            timer_driver.parallel_sleep(Duration::from_ms(100)).await?;
        }
    }
}

// Takes the 4 least-significant bits of x, and turn the 4 leds on/off accordingly.
fn blink_nibble(x: u8) -> TockResult<()> {
    for i in 0..4 {
        let led = led::get(i).unwrap();
        if (x >> i) & 1 != 0 {
            led.on()?;
        } else {
            led.off()?;
        }
    }
    Ok(())
}
