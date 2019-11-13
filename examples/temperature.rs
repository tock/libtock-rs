#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::temperature;
use libtock::timer;
use libtock_support_macros::generate_main;

#[generate_main]
async fn main() {
    let mut console = Console::new();

    loop {
        let result = temperature::measure_temperature().await;
        writeln!(console, "Temperature: {}", result).unwrap();
        timer::sleep(timer::Duration::from_ms(1000)).await;
    }
}
