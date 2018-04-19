#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate corepack;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tock;

use alloc::Vec;
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
        match ble_parser::find(&my_buffer, tock::simple_ble::gap_data::SERVICE_DATA as u8) {
            Some(payload) => {
                let payload: Vec<u8> = payload.into_iter().map(|x| *x).collect::<Vec<u8>>();
                let msg: LedCommand = corepack::from_bytes(payload.as_slice()).unwrap();
                let msg_led = led::get(msg.nr as isize);
                match msg_led {
                    Some(msg_led) => match msg.st {
                        true => msg_led.on(),
                        false => msg_led.off(),
                    },
                    _ => (),
                }
            }
            None => (),
        }
    });

    let _subscription = BleDriver::start(&mut callback);

    loop {
        syscalls::yieldk();
    }

    _subscription.unwrap();
}
