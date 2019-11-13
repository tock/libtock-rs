#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::libtock_main;
use libtock::temperature;
use libtock::timer;

#[libtock_main]
async fn main() {
    let mut console = Console::new();

    loop {
        let result = temperature::measure_temperature().await;
        writeln!(console, "Temperature: {}", result).unwrap();
        timer::sleep(timer::Duration::from_ms(1000)).await;
    }
}
