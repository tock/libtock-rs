#![feature(alloc)]
#![no_std]

extern crate tock;
extern crate alloc;

use tock::{led, timer};

fn main() {
    let led_count = alloc::boxed::Box::new(led::count());
    loop {
        for i in 0..*led_count {
            led::toggle(i as u32);
            timer::delay_ms(500);
        }
    }
}
