#![no_std]

use core::fmt::Write;
use libtock::gpio::InputMode;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Drivers;

// example works on p0.03
#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        console_driver,
        timer_context,
        gpio_driver,
        ..
    } = libtock::retrieve_drivers()?;
    let mut console = console_driver.create_console();
    let pin = gpio_driver.pin(0)?;
    let pin = pin.open_for_read(None, InputMode::PullDown)?;
    let mut driver = timer_context.create_timer_driver();
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
