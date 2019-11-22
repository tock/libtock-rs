#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::temperature;

async fn main() {
    let mut console = Console::new();
    let temperature = temperature::measure_temperature().await;
    writeln!(console, "Temperature: {}", temperature).unwrap();
}
