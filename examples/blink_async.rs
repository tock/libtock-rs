#![no_std]

extern crate tock;

use tock::led;
use tock::timer;

extern "C" fn timer_event(_: usize, _: usize, _: usize, _: usize) {
    led::get(0).unwrap().toggle();
}

fn main() {
    unsafe {
        timer::subscribe(timer_event, 0);
    }
    timer::start_repeating(500);
}
