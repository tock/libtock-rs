#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate corepack;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tock;

use alloc::String;
use tock::ble_composer;
use tock::ble_composer::BlePayload;
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
    let uuid: [u8; 2] = [0x00, 0x18];

    let payload = corepack::to_bytes(LedCommand { nr: 2, st: true }).unwrap();

    let mut buffer = BleAdvertisingDriver::create_advertising_buffer();
    let mut gap_payload = BlePayload::new();
    gap_payload.add_flag(ble_composer::flags::LE_GENERAL_DISCOVERABLE);

    gap_payload.add(ble_composer::gap_types::UUID, &uuid);

    gap_payload.add(
        ble_composer::gap_types::COMPLETE_LOCAL_NAME,
        name.as_bytes(),
    );
    gap_payload.add_service_payload([91, 79], &payload);

    let handle = BleAdvertisingDriver::initialize(100, &gap_payload, &mut buffer).unwrap();

    loop {
        led.on();
        timer::sleep(Duration::from_ms(500));
        led.off();;
        timer::sleep(Duration::from_ms(500));
    }
}
