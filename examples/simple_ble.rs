#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate corepack;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tock;

use alloc::String;
use tock::led;
use tock::simple_ble::BleAdvertisingDriver;
use tock::timer;
use tock::timer::Duration;

#[derive(Serialize)]
struct LedCommand {
    pub nr: u8,
    pub st: bool,
}

#[allow(unused_variables)]
fn main() {
    let led = led::get(0).unwrap();

    let name = String::from("Tock!");
    let uuid: [u16; 1] = [0x0018];

    let payload = corepack::to_bytes(LedCommand { nr: 2, st: true }).unwrap();

    let handle = BleAdvertisingDriver::initialize(100, name, uuid.to_vec(), true, payload).unwrap();

    loop {
        led.on();
        timer::sleep(Duration::from_ms(500));
        led.off();;
        timer::sleep(Duration::from_ms(500));
    }
}
