#![no_std]

use core::fmt::Write;
use tock::console::Console;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut console = Console::new();

    for i in 0.. {
        writeln!(console, "Hello world! {}", i).unwrap();
        timer::sleep(Duration::from_ms(500))
    }
}
