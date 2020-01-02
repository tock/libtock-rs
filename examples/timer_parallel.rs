#![no_std]

use futures::future;
use libtock::led::Led;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::timer::ParallelSleepDriver;
use libtock::Drivers;

async fn blink<'a>(
    timer_driver: &'a ParallelSleepDriver<'a>,
    duration: Duration<usize>,
    led: &'a mut Led<'a>,
) -> TockResult<()> {
    loop {
        led.toggle()?;

        timer_driver.sleep(duration).await?;
    }
}

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        led_driver,
        timer_context,
        ..
    } = libtock::retrieve_drivers()?;
    let mut led_iter = led_driver.all();
    let mut led_1 = led_iter.next().unwrap();
    let mut led_2 = led_iter.next().unwrap();
    let mut led_3 = led_iter.next().unwrap();

    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;

    let fut_1 = blink(&timer_driver, Duration::from_ms(500), &mut led_1);
    let fut_2 = blink(&timer_driver, Duration::from_ms(333), &mut led_2);
    let fut_3 = blink(&timer_driver, Duration::from_ms(250), &mut led_3);

    future::try_join3(fut_1, fut_2, fut_3).await?;
    Ok(())
}
