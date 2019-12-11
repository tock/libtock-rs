#![no_std]

use core::fmt::Write;
use futures::future;
use libtock::console::Console;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;

async fn main() -> TockResult<()> {
    let mut console = Console::new();

    let mut with_callback = timer::with_callback(|_, _| {
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
