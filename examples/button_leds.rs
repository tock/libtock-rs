#![no_std]

use futures::future;
use libtock::buttons::ButtonState;
use libtock::result::TockResult;
use libtock::Drivers;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        mut buttons_driver_factory,
        mut leds_driver_factory,
        ..
    } = libtock::retrieve_drivers()?;

    let buttons_driver = buttons_driver_factory.init_driver()?;
    let leds_driver = leds_driver_factory.init_driver()?;

    let mut callback = |button_num, state| {
        if let (ButtonState::Pressed, Some(led)) = (state, leds_driver.get(button_num)) {
            led.toggle().ok().unwrap();
        }
    };

    let _subscription = buttons_driver.subscribe(&mut callback)?;
    for button in buttons_driver.buttons() {
        button.enable_interrupt()?;
    }

    future::pending().await
}
