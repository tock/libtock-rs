#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate tock;

use alloc::String;
use tock::led;
use tock::simple_ble::BleDeviceUninitialized;
use tock::timer;

fn main() {
    let led = led::get(0).unwrap();

    let name = String::from("Hello from Tock");
    let uuid: [u16; 1] = [0x1800];
    let mut bleuninit = BleDeviceUninitialized::new(100, name, uuid.to_vec(), true, 1234);
    let mut bleinit = bleuninit.initialize().unwrap();
    bleinit.start_advertising().unwrap();

    loop {
        led.on();
        timer::delay_ms(500);
        led.off();;
        timer::delay_ms(500);
    }
}
