#![no_std]

use libtock::electronics::ShiftRegister;
use libtock::gpio::GpioPinUnitialized;
use libtock::timer;
use libtock::timer::Duration;

fn number_to_bits(n: u8) -> [bool; 8] {
    match n {
        1 => [false, false, false, true, false, true, false, false],
        2 => [true, false, true, true, false, false, true, true],
        3 => [true, false, true, true, false, true, true, false],
        4 => [true, true, false, true, false, true, false, false],
        5 => [true, true, true, false, false, true, true, false],
        6 => [true, true, true, false, false, true, true, true],
        7 => [false, false, true, true, false, true, false, false],
        8 => [true, true, true, true, false, true, true, true],
        9 => [true, true, true, true, false, true, true, false],
        0 => [false, true, true, true, false, true, true, true],
        _ => [false, false, false, false, true, false, false, false],
    }
}

// Example works on a shift register on P0.03, P0.04, P0.28
fn main() {
    let shift_register = ShiftRegister::new(
        GpioPinUnitialized::new(0).open_for_write().unwrap(),
        GpioPinUnitialized::new(1).open_for_write().unwrap(),
        GpioPinUnitialized::new(2).open_for_write().unwrap(),
    );

    let mut i = 0;
    loop {
        i = (i + 1) % 11;
        shift_register.write_bits(&number_to_bits(i));
        timer::sleep(Duration::from_ms(200));
    }
}
