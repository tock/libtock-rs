#![no_std]

extern crate tock;

use tock::gpio::GpioPinUnitialized;
use tock::timer;
use tock::timer::Duration;

// Example works on P0.03
fn main() {
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_write().unwrap();

    loop {
        pin.set_high();
        timer::sleep(Duration::from_ms(500));
        pin.set_low();;
        timer::sleep(Duration::from_ms(500));
    }
}
