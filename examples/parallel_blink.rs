#![no_std]

use futures::future;
use libtock::led;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;
use libtock::timer::ParallelSleepDriver;

async fn blink<'a>(
    timer_driver: &'a ParallelSleepDriver<'a>,
    duration: Duration<usize>,
    led_number: usize,
) -> TockResult<()> {
    loop {
        led::get(led_number).unwrap().toggle()?;

        timer_driver.sleep(duration).await?;
    }
}

#[libtock::main]
async fn main() -> TockResult<()> {
    let context = timer::DriverContext::create()?;
    let mut driver = context.create_timer_driver().unwrap();
    let timer_driver = driver.activate()?;

    let fut_1 = blink(&timer_driver, Duration::from_ms(500), 0);
    let fut_2 = blink(&timer_driver, Duration::from_ms(333), 1);
    let fut_3 = blink(&timer_driver, Duration::from_ms(250), 2);

    future::try_join(future::try_join(fut_1, fut_2), fut_3).await?;
    Ok(())
}
