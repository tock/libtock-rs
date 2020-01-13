#![no_std]

use core::fmt::Write;
use libtock::gpio::ResistorMode;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Drivers;

// example works on p0.03
#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        mut gpio_driver_factory,
        mut timer_context,
        console_driver,
        ..
    } = libtock::retrieve_drivers()?;

    let mut gpio_driver = gpio_driver_factory.init_driver()?;
    let mut timer_driver = timer_context.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    let mut console = console_driver.create_console();

    let mut gpio = gpio_driver.gpios().next().unwrap();
    let gpio_in = gpio.enable_input(ResistorMode::PullDown)?;
    loop {
        writeln!(console, "{:?}", gpio_in.read()?)?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
