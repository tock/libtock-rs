//! An extremely simple libtock-rs example. Just prints out a few numbers using
//! the LowLevelDebug capsule then terminates.

#![no_main]
#![no_std]

use libtock2::leds::LedsFactory;
use libtock2::runtime::{set_main, stack_size};
use libtock_platform::ErrorCode;

set_main! {main}
stack_size! {0x100}

fn main() -> Result<(), ErrorCode> {
    // placeholder until a driver infrastrucrure is built
    let mut leds_factory = LedsFactory::new();

    let leds_driver = leds_factory.init_driver()?;
    for led in leds_driver.leds() {
        led.on()?
    }

    for led in leds_driver.leds() {
        led.off()?
    }
    leds_driver.get(0)?.on()
}
