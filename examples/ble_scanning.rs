#![no_std]

extern crate corepack;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tock;

use tock::ble_parser;
use tock::led;
use tock::simple_ble::BleCallback;
use tock::simple_ble::BleDriver;
use tock::syscalls;

#[derive(Deserialize)]
struct LedCommand {
    pub nr: u8,
    pub st: bool,
}

// Prevents the compiler from dropping the subscription too early.
#[allow(unreachable_code)]
fn main() {
    let mut shared_buffer = BleDriver::create_scan_buffer();
    let mut my_buffer = BleDriver::create_scan_buffer();
    let shared_memory = BleDriver::share_memory(&mut shared_buffer).unwrap();

    let mut callback = BleCallback::new(|_: usize, _: usize| {
        shared_memory.read_bytes(&mut my_buffer);
        ble_parser::find(&my_buffer, tock::simple_ble::gap_data::SERVICE_DATA as u8)
            .and_then(|service_data| ble_parser::extract_for_service([91, 79], service_data))
            .and_then(|payload| corepack::from_bytes::<LedCommand>(&payload).ok())
            .and_then(|msg| led::get(msg.nr as isize).map(|led| led.set_state(msg.st)));
    });

    let _subscription = BleDriver::start(&mut callback).unwrap();

    loop {
        syscalls::yieldk();
    }
}
