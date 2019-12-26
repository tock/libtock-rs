#![no_std]

use libtock::led::LedDriver;
use libtock::result::TockResult;
use libtock::rng;
use libtock::timer::Duration;
use libtock::Hardware;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware { timer_context, .. } = libtock::retrieve_hardware()?;

    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;
    let Hardware { mut led_driver, .. } = libtock::retrieve_hardware()?;

    let num_leds = led_driver.count()?;
    // blink_nibble assumes 4 leds.
    assert_eq!(num_leds, 4);

    let mut buf = [0; 64];
    loop {
        rng::fill_buffer(&mut buf).await?;

        for &x in buf.iter() {
            blink_nibble(x, &mut led_driver)?;
            timer_driver.sleep(Duration::from_ms(100)).await?;
            blink_nibble(x >> 4, &mut led_driver)?;
            timer_driver.sleep(Duration::from_ms(100)).await?;
        }
    }
}

// Takes the 4 least-significant bits of x, and turn the 4 leds on/off accordingly.
fn blink_nibble(x: u8, led_driver: &mut LedDriver) -> TockResult<()> {
    for i in 0..4 {
        let mut led = led_driver.get(i).unwrap();
        if (x >> i) & 1 != 0 {
            led.on()?;
        } else {
            led.off()?;
        }
    }
    Ok(())
}
