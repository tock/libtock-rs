#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::syscalls;
use libtock::temperature;

fn main() {
    let mut console = Console::new();

    let mut with_callback = temperature::with_callback(|result: isize| {
        writeln!(console, "Temperature: {}", result).unwrap();
    });

    let _temperature = with_callback.start_measurement();

    loop {
        syscalls::yieldk();
    }
}
