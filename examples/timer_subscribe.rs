#![no_std]

use core::fmt::Write;
use futures::future;
use libtock::result::TockResult;
use libtock::timer::Duration;
use libtock::Drivers;

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        mut timer_context,
        console_driver,
        ..
    } = libtock::retrieve_drivers()?;

    let mut console = console_driver.create_console();

    let mut with_callback = timer_context.with_callback(|_, _| {
        writeln!(
            console,
            "This line is printed 2 seconds after the start of the program.",
        )
        .unwrap();
    });

    let mut timer = with_callback.init()?;
    timer.set_alarm(Duration::from_ms(2000))?;

    future::pending().await
}
