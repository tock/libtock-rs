#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::timer;
use libtock::timer::Duration;

fn main() {
    let mut console = Console::new();

    for i in 0.. {
        writeln!(console, "Hello world! {}", i).unwrap();
        timer::sleep(Duration::from_ms(500))
    }
}
