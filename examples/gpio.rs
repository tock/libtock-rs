#![no_std]

use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Hardware;

// Example works on P0.03
#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware {
        timer_context,
        mut gpio_driver,
        ..
    } = libtock::retrieve_hardware()?;
    let pin = gpio_driver.pin(0)?;
    let mut pin = pin.open_for_write()?;
    let mut driver = timer_context.create_timer_driver();
    let timer_driver = driver.activate()?;

    loop {
        pin.set_high()?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
        pin.set_low()?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
