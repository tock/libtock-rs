#![no_std]

use core::executor;
use core::fmt::Write;
use libtock::console::Console;
use libtock::temperature;
use libtock::timer;

async fn main() {
    let mut console = Console::new();

    loop {
        let result = executor::block_on(async {
            timer::sleep(timer::Duration::from_ms(1000)).await;
            temperature::measure_temperature().await
        });

        writeln!(console, "Temperature: {}", result).unwrap();
    }
}
