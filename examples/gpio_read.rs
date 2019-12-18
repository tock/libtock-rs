#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::gpio::GpioPinUnitialized;
use libtock::gpio::InputMode;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;

// example works on p0.03
#[libtock::main]
async fn main() -> TockResult<()> {
    let mut console = Console::new();
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_read(None, InputMode::PullDown)?;
    let context = timer::DriverContext::create()?;
    let mut driver = context.create_timer_driver().unwrap();
    let timer_driver = driver.activate()?;

    loop {
        if pin.read() {
            writeln!(console, "true")?;
        } else {
            writeln!(console, "false")?;
        }
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
