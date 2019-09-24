#![no_std]

use libtock::ble_composer;
use libtock::ble_composer::BlePayload;
use libtock::led;
use libtock::simple_ble::BleAdvertisingDriver;
use libtock::timer;
use libtock::timer::Duration;
use serde::Serialize;

#[derive(Serialize)]
struct LedCommand {
    pub nr: u8,
    pub st: bool,
}

#[allow(unused_variables)]
fn main() {
    let led = led::get(0).unwrap();

    let uuid: [u8; 2] = [0x00, 0x18];

    let payload = corepack::to_bytes(LedCommand { nr: 2, st: true }).unwrap();

    let mut buffer = BleAdvertisingDriver::create_advertising_buffer();
    let mut gap_payload = BlePayload::new();
    gap_payload
        .add_flag(ble_composer::flags::LE_GENERAL_DISCOVERABLE)
        .unwrap();

    gap_payload
        .add(ble_composer::gap_types::UUID, &uuid)
        .unwrap();

    gap_payload
        .add(
            ble_composer::gap_types::COMPLETE_LOCAL_NAME,
            "Tock!".as_bytes(),
        )
        .unwrap();
    gap_payload.add_service_payload([91, 79], &payload).unwrap();

    let handle = BleAdvertisingDriver::initialize(100, &gap_payload, &mut buffer).unwrap();

    loop {
        led.on();
        timer::sleep(Duration::from_ms(500));
        led.off();
        timer::sleep(Duration::from_ms(500));
    }
}
