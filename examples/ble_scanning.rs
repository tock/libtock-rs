#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate tock;

// Macro usages are not detected
#[allow(unused_imports)]
use alloc::*;
use tock::ble_parser;
use tock::led;
use tock::syscalls::yieldk;

fn main() {
    let led = led::get(0).unwrap();

    let buffer = [0; tock::simple_ble::BUFFER_SIZE_SCAN];
    tock::simple_ble::BleScanning::start(&buffer, |_: usize, _: usize| {
        if ble_parser::find(&buffer, 0xFF) == Some(vec![&0xFF, &0xFF, &0xFF, &0xFF]) {
            led.on();
        }
        if ble_parser::find(&buffer, 0xFF) == Some(vec![&0xFF, &0xFF, &0x00, &0x00]) {
            led.off();
        }
    }).unwrap();

    loop {
        yieldk();
    }
}
