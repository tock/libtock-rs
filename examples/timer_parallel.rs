#![no_std]

use futures::future;
use libtock::alarm::Duration;
use libtock::alarm::ParallelSleepDriver;
use libtock::leds::Led;
use libtock::result::TockResult;

libtock_core::stack_size! {0x800}

async fn blink(
    timer_driver: &ParallelSleepDriver<'_>,
    duration: Duration<usize>,
    led: Led<'_>,
) -> TockResult<()> {
    loop {
        led.toggle()?;

        timer_driver.sleep(duration).await?;
    }
}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let leds_driver = drivers.leds.init_driver()?;
    let mut timer_driver = drivers.alarm.create_timer_driver();
    let timer_driver = timer_driver.activate()?;

    let mut leds = leds_driver.leds();

    let fut_1 = blink(&timer_driver, Duration::from_ms(500), leds.next().unwrap());
    let fut_2 = blink(&timer_driver, Duration::from_ms(333), leds.next().unwrap());
    let fut_3 = blink(&timer_driver, Duration::from_ms(250), leds.next().unwrap());

    future::try_join3(fut_1, fut_2, fut_3).await?;
    Ok(())
}
