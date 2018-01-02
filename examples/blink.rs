#![no_std]

extern crate tock;

use tock::led;
use tock::timer;

fn main() {
    let led = led::get(0).unwrap();
    loop {
        led.on();
        timer::delay_ms(500);
        led.off();;
        timer::delay_ms(500);
    }
}
