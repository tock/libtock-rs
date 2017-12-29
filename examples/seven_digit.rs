#![no_std]

extern crate tock;

const DATA_PIN: u32 = 0;
const CLOCK_PIN: u32 = 1;
const LATCH_PIN: u32 = 2;
const SPEED: u32 = 2;

use tock::{led, timer};

fn push_one_bit(value: bool) {
    if value {
        led::on(DATA_PIN)
    } else {
        led::off(DATA_PIN)
    }
    led::on(CLOCK_PIN);
    timer::delay_ms(SPEED);
    led::off(CLOCK_PIN);
    timer::delay_ms(SPEED);
}

fn push_bits(values: &[bool]) {
    for i in values {
        push_one_bit(*i);
    }
    display();
    timer::delay_ms(200);
}

fn display() {
    led::on(LATCH_PIN);
    timer::delay_ms(SPEED);
    led::off(LATCH_PIN);
}

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
    let mut i = 0;
    loop {
        i = (i + 1) % 11;
        push_bits(&number_to_bits(i));
    }
}
