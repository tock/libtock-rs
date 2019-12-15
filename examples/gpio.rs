#![no_std]

use libtock::gpio::GpioPinUnitialized;
use libtock::timer;
use libtock::timer::Duration;
use libtock_support_macros::libtock_main;

// Example works on P0.03
#[libtock_main]
async fn main() -> libtock::result::TockResult<()> {
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_write()?;

    loop {
        pin.set_high()?;
        timer::sleep(Duration::from_ms(500)).await?;
        pin.set_low()?;
        timer::sleep(Duration::from_ms(500)).await?;
    }
}
