#![no_std]

extern crate tock;

use tock::led;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let led = led::get(0).unwrap();

    loop {
        led.toggle();
        timer::sleep(Duration::from_ms(500));
    }
}
