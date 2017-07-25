#![no_std]

extern crate tock;

use tock::{led, timer};

extern fn timer_event(_: usize, _: usize, _: usize, _: usize) {
    led::toggle(0);
}

fn main() {
    unsafe {
        timer::subscribe(timer_event, 0);
    }
    timer::start_repeating(500);
}
