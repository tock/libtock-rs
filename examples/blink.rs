#![no_std]
#![no_main]

#[macro_use]
extern crate tock;

use tock::{led, timer};

tock_main!({
    let led_count = led::count();
    loop {
        for i in 0..led_count {
            led::toggle(i as u32);
            timer::delay_ms(500);
        }
    }
});

