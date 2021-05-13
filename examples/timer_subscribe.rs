#![no_std]

use futures::future;
use libtock::alarm::Duration;
use libtock::println;
use libtock::result::TockResult;

libtock_core::stack_size! {0x800}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    drivers.console.create_console();

    let mut with_callback = drivers.alarm.with_callback(|_, _| {
        println!("This line is printed 2 seconds after the start of the program.");
    });

    let mut timer = with_callback.init()?;
    timer.set_alarm(Duration::from_ms(2000))?;

    future::pending().await
}
