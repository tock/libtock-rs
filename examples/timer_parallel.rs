#![no_std]

use futures::future;
use libtock::leds::Led;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::timer::ParallelSleepDriver;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x800] = [0; 0x800];

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
    let mut timer_driver = drivers.timer.create_timer_driver();
    let timer_driver = timer_driver.activate()?;

    let mut leds = leds_driver.leds();

    let fut_1 = blink(&timer_driver, Duration::from_ms(500), leds.next().unwrap());
    let fut_2 = blink(&timer_driver, Duration::from_ms(333), leds.next().unwrap());
    let fut_3 = blink(&timer_driver, Duration::from_ms(250), leds.next().unwrap());

    future::try_join3(fut_1, fut_2, fut_3).await?;
    Ok(())
}
