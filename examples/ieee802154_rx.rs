//! An example showing use of IEEE 802.15.4 networking.
//! It infinitely received a frame and prints its content to Console.

#![no_main]
#![no_std]
use core::fmt::Write as _;
use libtock::console::Console;
use libtock::ieee802154::{Ieee802154, RxOperator as _, RxRingBuffer, RxSingleBufferOperator};
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x600}

fn main() {
    // Configure the radio
    let pan: u16 = 0xcafe;
    let addr_short: u16 = 0xdead;
    let addr_long: u64 = 0xdead_dad;
    let tx_power: i8 = 5;
    let channel: u8 = 11;

    Ieee802154::set_pan(pan);
    Ieee802154::set_address_short(addr_short);
    Ieee802154::set_address_long(addr_long);
    Ieee802154::set_tx_power(tx_power).unwrap();
    Ieee802154::set_channel(channel).unwrap();

    // Don't forget to commit the config!
    Ieee802154::commit_config();

    // Turn the radio on
    Ieee802154::radio_on().unwrap();
    assert!(Ieee802154::is_on());

    let mut buf = RxRingBuffer::<2>::new();
    let mut operator = RxSingleBufferOperator::new(&mut buf);
    loop {
        let frame = operator.receive_frame().unwrap();

        let body_len = frame.payload_len;
        writeln!(
            Console::writer(),
            "Received frame with body of len {}: {} {:?}!\n",
            body_len,
            core::str::from_utf8(&frame.body).unwrap(),
            &frame.body[..frame.body.len() - core::mem::size_of::<usize>()]
        )
        .unwrap();
    }
}
