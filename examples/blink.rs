#![no_std]

extern crate tock;

use tock::{led, timer};

fn main() {
    loop{
        led::on(0);
        timer::delay_ms(500);
        led::off(0);
        timer::delay_ms(500);
    }
}
