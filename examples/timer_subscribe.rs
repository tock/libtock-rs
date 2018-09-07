#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::console::Console;
use tock::syscalls;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut console = Console::new();

    let mut with_callback = timer::with_callback(|_, _| {
        console.write(String::from(
            "This line is printed 2 seconds after the start of the program.",
        ))
    });

    let mut timer = with_callback.init().unwrap();

    timer.set_alarm(Duration::from_ms(2000)).unwrap();
    loop {
        syscalls::yieldk();
    }
}
