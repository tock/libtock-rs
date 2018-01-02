#![no_std]

extern crate tock;

use tock::electronics::ShiftRegister;
use tock::led;
use tock::timer;

fn number_to_bits(n: u8) -> [bool; 8] {
    match n {
        1 => [true, true, true, false, true, false, true, true],
        2 => [false, true, false, false, true, true, false, false],
        3 => [false, true, false, false, true, false, false, true],
        4 => [false, false, true, false, true, false, true, true],
        5 => [false, false, false, true, true, false, false, true],
        6 => [false, false, false, true, true, false, false, false],
        7 => [true, true, false, false, true, false, true, true],
        8 => [false, false, false, false, true, false, false, false],
        9 => [false, false, false, false, true, false, false, true],
        0 => [true, false, false, false, true, false, false, false],
        _ => [true, true, true, true, false, true, true, true],
    }
}

fn main() {
    let shift_register = ShiftRegister::new(
        led::get(0).unwrap(),
        led::get(1).unwrap(),
        led::get(2).unwrap(),
    );

    let mut i = 0;
    loop {
        i = (i + 1) % 11;
        shift_register.write_bits(&number_to_bits(i));
        timer::delay_ms(200);
    }
}
