#![feature(alloc)]
#![no_std]

extern crate alloc;

use alloc::string::String;
use tock::console::Console;
use tock::temperature;

fn main() {
    let mut console = Console::new();

    let mut with_callback = temperature::with_callback(|result: isize| {
        console.write(String::from("Temperature: "));
        console.write(tock::fmt::i32_as_decimal(result as i32));
        console.write(String::from("\n"));
    });

    let _temperature = with_callback.start_measurement();

    loop {
        tock::syscalls::yieldk();
    }
}
