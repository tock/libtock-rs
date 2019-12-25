#![no_std]

use core::fmt::Write;
use libtock::gpio::GpioPinUnitialized;
use libtock::gpio::InputMode;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;
use libtock::Hardware;

// example works on p0.03
#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware { console_driver } = libtock::retrieve_hardware()?;
    let mut console = console_driver.create_console();
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_read(None, InputMode::PullDown)?;
    let context = timer::DriverContext::create()?;
    let mut driver = context.create_timer_driver()?;
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
