#![no_std]

use core::cell::Cell;
use core::fmt::Write;
use libtock::buttons::ButtonState;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Drivers;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        mut buttons_driver_factory,
        mut timer_context,
        console_driver,
        ..
    } = libtock::retrieve_drivers()?;

    let buttons_driver = buttons_driver_factory.init_driver()?;
    let mut timer_driver = timer_context.create_timer_driver();
    let timer_driver = timer_driver.activate()?;
    let mut console = console_driver.create_console();

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
        writeln!(
            console,
            "pressed: {}, released: {}",
            pressed_count.get(),
            released_count.get()
        )?;
        timer_driver.sleep(Duration::from_ms(500)).await?;
    }
}
