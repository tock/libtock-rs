#![no_std]

use libtock::alarm::Duration;
use libtock::leds::LedsDriver;
use libtock::result::TockResult;

libtock_core::stack_size! {0x400}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let leds_driver = drivers.leds.init_driver()?;
    let mut timer_driver = drivers.alarm.create_timer_driver();
    let timer_driver = timer_driver.activate()?;

    // blink_nibble assumes 4 leds.
    assert_eq!(leds_driver.num_leds(), 4);

    let mut buf = [0; 64];
    loop {
        drivers.rng.fill_buffer(&mut buf).await?;

        for &x in buf.iter() {
            blink_nibble(&leds_driver, x)?;
            timer_driver.sleep(Duration::from_ms(100)).await?;
            blink_nibble(&leds_driver, x >> 4)?;
            timer_driver.sleep(Duration::from_ms(100)).await?;
        }
    }
}

// Takes the 4 least-significant bits of x, and turn the 4 leds on/off accordingly.
fn blink_nibble(leds_driver: &LedsDriver, x: u8) -> TockResult<()> {
    for i in 0..4 {
        let led = leds_driver.get(i)?;
        if (x >> i) & 1 != 0 {
            led.on()?;
        } else {
            led.off()?;
        }
    }
    Ok(())
}
