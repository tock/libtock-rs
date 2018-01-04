#![no_std]

extern crate tock;

use tock::gpio::GpioPinUnitialized;
use tock::timer;

// Example works on P0.03
fn main() {
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_write().unwrap();

    loop {
        pin.set_high();
        timer::delay_ms(500);
        pin.set_low();;
        timer::delay_ms(500);
    }
}
