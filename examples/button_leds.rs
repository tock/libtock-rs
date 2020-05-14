#![no_std]

use futures::future;
use libtock::buttons::ButtonState;
use libtock::result::TockResult;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x800] = [0; 0x800];

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let buttons_driver = drivers.buttons.init_driver()?;
    let leds_driver = drivers.leds.init_driver()?;

    let mut callback = |button_num, state| {
        if let (ButtonState::Pressed, Ok(led)) = (state, leds_driver.get(button_num)) {
            led.toggle().ok().unwrap();
        }
    };

    let _subscription = buttons_driver.subscribe(&mut callback)?;
    for button in buttons_driver.buttons() {
        button.enable_interrupt()?;
    }

    future::pending().await
}
