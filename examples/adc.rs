#![no_std]

use core::fmt::Write;
use libtock::adc;
use libtock::console::Console;
use libtock::timer;
use libtock::timer::Duration;

fn main() {
    let mut console = Console::new();
    let mut with_callback = adc::with_callback(|channel: usize, value: usize| {
        writeln!(console, "channel: {}, value: {}", channel, value).unwrap();
    });

    let adc = with_callback.init().unwrap();

    loop {
        adc.sample(0).unwrap();
        timer::sleep(Duration::from_ms(2000));
    }
}
