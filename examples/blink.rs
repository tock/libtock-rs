//! The blink app.

#![no_main]
#![no_std]

use libtock::alarm::{Alarm, Milliseconds};
use libtock::leds::Leds;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x200}

fn main() {
    let mut count = 0;
    if let Ok(leds_count) = Leds::count() {
        loop {
            for led_index in 0..leds_count {
                if count & (1 << led_index) > 0 {
                    let _ = Leds::on(led_index as u32);
                } else {
                    let _ = Leds::off(led_index as u32);
                }
            }

            Alarm::sleep_for(Milliseconds(250)).unwrap();

            count += 1;
        }
    }
}
