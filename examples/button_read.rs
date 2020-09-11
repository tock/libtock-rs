#![no_std]

use libtock::println;
use libtock::result::TockResult;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let buttons_driver = drivers.buttons.init_driver()?;
    let mut timer_driver = drivers.timer.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    drivers.console.create_console();

    loop {
        for button in buttons_driver.buttons() {
            println!("button {}: {:?}", button.button_num(), button.read()?);
        }
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
