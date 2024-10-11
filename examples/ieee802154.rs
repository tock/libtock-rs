//! An example showing use of IEEE 802.15.4 networking.

#![no_main]
#![no_std]
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
    let tx_power: i8 = -3;
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

    // Transmit a frame
    Ieee802154::transmit_frame(b"foobar").unwrap();

    Console::write(b"Transmitted frame!\n").unwrap();

    // Showcase receiving to a single buffer - there is a risk of losing some frames.
    // See [RxSingleBufferOperator] docs for more details.
    rx_single_buffer();
}

fn rx_single_buffer() {
    let mut buf = RxRingBuffer::<2>::new();
    let mut operator = RxSingleBufferOperator::new(&mut buf);

    let frame1 = operator.receive_frame().unwrap();
    // Access frame1 data here:
    let _body_len = frame1.payload_len;
    let _first_body_byte = frame1.body[0];

    let _frame2 = operator.receive_frame().unwrap();
    // Access frame2 data here
}
