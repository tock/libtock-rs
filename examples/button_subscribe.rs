#![no_std]

use core::cell::Cell;
use libtock::buttons::ButtonState;
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

    let pressed_count = Cell::new(0usize);
    let released_count = Cell::new(0usize);

    let mut callback = |_button_num, state| match state {
        ButtonState::Pressed => pressed_count.set(pressed_count.get() + 1),
        ButtonState::Released => released_count.set(released_count.get() + 1),
    };

    let _subscription = buttons_driver.subscribe(&mut callback)?;

    for button in buttons_driver.buttons() {
        button.enable_interrupt()?;
    }

    loop {
        println!(
            "pressed: {}, released: {}",
            pressed_count.get(),
            released_count.get()
        );
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
