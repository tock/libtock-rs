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

fn main() {
    let buffer = [0; tock::simple_ble::BUFFER_SIZE_SCAN];
    BleDriver::start(&buffer, |_: usize, _: usize| {
        match ble_parser::find(&buffer, tock::simple_ble::gap_data::SERVICE_DATA as u8) {
            Some(payload) => {
                let payload: Vec<u8> = payload.iter().map(|&x| *x).collect::<Vec<u8>>();
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
    }).unwrap();

    loop {
        syscalls::yieldk();
    }
}
