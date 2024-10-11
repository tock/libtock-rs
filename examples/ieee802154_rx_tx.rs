//! An example showing use of IEEE 802.15.4 networking.
//! It infinitely sends a frame with a constantly incremented counter,
//! and after each send receives a frame and prints it to Console.

#![no_main]
#![no_std]
use core::fmt::Write as _;
use libtock::alarm::{Alarm, Milliseconds};
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

    let mut counter = 0_usize;
    let mut buf = [
        b'f', b'r', b'a', b'm', b'e', b' ', b'n', b'.', b'o', b'.', b' ', b'\0', b'\0', b'\0',
        b'\0',
    ];
    fn set_buf_cnt(buf: &mut [u8], counter: &mut usize) {
        let buf_len = buf.len();
        let buf_cnt = &mut buf[buf_len - core::mem::size_of_val(&counter)..];
        buf_cnt.copy_from_slice(&counter.to_be_bytes());
    }

    loop {
        Alarm::sleep_for(Milliseconds(1000)).unwrap();

        set_buf_cnt(&mut buf, &mut counter);

        // Transmit a frame
        Ieee802154::transmit_frame(&buf).unwrap();

        writeln!(Console::writer(), "Transmitted frame {}!\n", counter).unwrap();

        let frame = operator.receive_frame().unwrap();

        let body_len = frame.payload_len;
        writeln!(
            Console::writer(),
            "Received frame with body of len {}: {}-{} {:?}!\n",
            body_len,
            core::str::from_utf8(&frame.body[..frame.body.len() - core::mem::size_of::<usize>()])
                .unwrap_or("<error decoding>"),
            usize::from_le_bytes(
                frame.body[frame.body.len() - core::mem::size_of::<usize>()..]
                    .try_into()
                    .unwrap()
            ),
            frame.body
        )
        .unwrap();

        counter += 1;
    }
}
