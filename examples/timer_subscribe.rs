#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::syscalls;
use libtock::timer;
use libtock::timer::Duration;

fn main() {
    let mut console = Console::new();

    let mut with_callback = timer::with_callback(|_, _| {
        writeln!(
            console,
            "This line is printed 2 seconds after the start of the program.",
        )
        .unwrap();
    });

    let mut timer = with_callback.init().unwrap();

    timer.set_alarm(Duration::from_ms(2000)).unwrap();
    loop {
        syscalls::yieldk();
    }
}
