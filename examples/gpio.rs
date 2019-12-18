#![no_std]

use libtock::gpio::GpioPinUnitialized;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;

// Example works on P0.03
#[libtock::main]
async fn main() -> TockResult<()> {
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_write()?;
    let context = timer::DriverContext::create()?;
    let mut driver = context.create_timer_driver().unwrap();
    let timer_driver = driver.activate()?;

    loop {
        pin.set_high()?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
        pin.set_low()?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
