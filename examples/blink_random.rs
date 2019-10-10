#![no_std]

use libtock::led;
use libtock::rng;
use libtock::timer;
use libtock::timer::Duration;

fn main() {
    let num_leds = led::count();
    // blink_nibble assumes 4 leds.
    assert_eq!(num_leds, 4);

    let mut buf = [0; 64];
    loop {
        assert!(rng::fill_buffer(&mut buf));

        for &x in buf.iter() {
            blink_nibble(x);
            timer::sleep(Duration::from_ms(100));
            blink_nibble(x >> 4);
            timer::sleep(Duration::from_ms(100));
        }
    }
}

// Takes the 4 least-significant bits of x, and turn the 4 leds on/off accordingly.
fn blink_nibble(x: u8) {
    for i in 0..4 {
        let led = led::get(i).unwrap();
        if (x >> i) & 1 != 0 {
            led.on();
        } else {
            led.off();
        }
    }
}
