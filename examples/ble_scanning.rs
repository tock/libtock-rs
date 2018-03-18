#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate corepack;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tock;

// Macro usages are not detected
#[allow(unused_imports)]
use alloc::*;
use tock::ble_parser;
use tock::led;
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
    let mut shared_memory = BleDriver::share_memory().unwrap();
    let _subscription = BleDriver::start(|_: usize, _: usize| {
        match ble_parser::find(
            shared_memory.to_bytes(),
            tock::simple_ble::gap_data::SERVICE_DATA as u8,
        ) {
            Some(payload) => {
                let payload: Vec<u8> = payload.into_iter().map(|x| *x).collect::<Vec<u8>>();
                let msg: LedCommand = corepack::from_bytes(payload.as_slice()).unwrap();
                let msg_led = led::get(msg.nr as isize);
                match msg_led {
                    Some(msg_led) => if msg.st {
                        msg_led.on();
                    } else {
                        msg_led.off();
                    },
                    _ => (),
                }
            }
            None => (),
        }
    });

    loop {
        syscalls::yieldk();
    }
    _subscription.unwrap();
}
