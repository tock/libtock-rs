#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate tock;

use alloc::{String, Vec};
use tock::led;
use tock::simple_ble::BleDeviceUninitialized;
use tock::timer;
use tock::timer::Duration;

#[allow(unused_variables)]
fn main() {
    let led = led::get(0).unwrap();

    let name = String::from("Hello from To");
    let uuid: [u16; 1] = [0x0018];
    let mut payload: Vec<u8> = Vec::new();
    payload.push(0x01);
    payload.push(0x02);

    let mut bleuninit = BleDeviceUninitialized::new(100, name, uuid.to_vec(), true, &mut payload);
    let mut bleinit = bleuninit.initialize().unwrap();
    let ble = bleinit.start_advertising().unwrap();

    loop {
        led.on();
        timer::sleep(Duration::from_ms(500));
        led.off();;
        timer::sleep(Duration::from_ms(500));
    }
}
