#![no_std]

extern crate tock;

use tock::gpio::{GpioPinUnitialized};
use tock::timer;

fn main() {
    let pin = GpioPinUnitialized::new(17);
    let pin = pin.open_for_write().unwrap();

    loop {
        pin.set_high();
        timer::delay_ms(500);
        pin.set_low();;
        timer::delay_ms(500);
    }
}
