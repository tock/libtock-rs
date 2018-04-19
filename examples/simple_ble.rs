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
    let mut uuid: [u8; 2] = [0x18, 0x00];

    let mut payload = corepack::to_bytes(LedCommand { nr: 2, st: true }).unwrap();

    let mut buffer = BleAdvertisingDriver::create_advertising_buffer();
    let handle =
        BleAdvertisingDriver::initialize(100, name, &mut uuid, true, &mut payload, &mut buffer)
            .unwrap();

    loop {
        led.on();
        timer::sleep(Duration::from_ms(500));
        led.off();;
        timer::sleep(Duration::from_ms(500));
    }
}
