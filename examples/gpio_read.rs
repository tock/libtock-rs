#![no_std]

use libtock::gpio::ResistorMode;
use libtock::println;
use libtock::result::TockResult;
use libtock::timer::Duration;

// example works on p0.03
#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let mut gpio_driver = drivers.gpio.init_driver()?;
    let mut timer_driver = drivers.timer.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    drivers.console.create_console();

    let mut gpio = gpio_driver.gpios().next().unwrap();
    let gpio_in = gpio.enable_input(ResistorMode::PullDown)?;
    loop {
        println!("{:?}", gpio_in.read()?);
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
