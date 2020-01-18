#![no_std]

use core::fmt::Write;
use futures::future;
use libtock::result::TockResult;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let mut console = drivers.console.create_console();

    let mut with_callback = drivers.timer.with_callback(|_, _| {
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
