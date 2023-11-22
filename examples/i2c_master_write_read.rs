//! This sample demonstrates setting up the i2c ip (assuming board has support)
//! for master mode. In the event loop, we write some bytes to the target, then
//! attempt to read some bytes from the target.
//!
//! This sample is tested with `i2c_slave_send_recv.rs` sample running on the
//! slave device. That sample uses the synchronous slave api, so the order of operations
//! is important to ensure we don't cause the slave to stretch clocks if it hasn't setup
//! send buffers in time.

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::console::Console;
use libtock::i2c_master_slave::I2CMasterSlave;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x400}

pub const SLAVE_DEVICE_ADDR: u16 = 0x69;

fn main() {
    let addr = SLAVE_DEVICE_ADDR;
    // 7-bit addressing
    assert!(addr <= 0x7f);
    let mut tx_buf: [u8; 4] = [0; 4];
    // Write 4 bytes to the slave
    let tx_len = 4;
    let mut rx_buf: [u8; 2] = [0; 2];
    // Attempt to read 2 bytes from the slave
    let rx_len = 2;

    writeln!(Console::writer(), "i2c-master: write-read sample\r").unwrap();
    writeln!(
        Console::writer(),
        "i2c-master: slave address 0x{:x}!\r",
        addr
    )
    .unwrap();

    let mut i: u32 = 0;
    loop {
        writeln!(
            Console::writer(),
            "i2c-master: write-read operation {:?}\r",
            i
        )
        .unwrap();

        // Change up the data in tx-buffer
        tx_buf[0] = tx_buf[0].wrapping_add(2);
        tx_buf[1] = tx_buf[1].wrapping_add(4);
        tx_buf[2] = tx_buf[2].wrapping_add(6);
        tx_buf[3] = tx_buf[3].wrapping_add(8);

        if let Err(why) = I2CMasterSlave::i2c_master_slave_write_sync(addr, &tx_buf, tx_len) {
            writeln!(
                Console::writer(),
                "i2c-master: write operation failed {:?}",
                why
            )
            .unwrap();
        } else {
            // This sample target the i2c_slave_send_recv.rs sample, which is synchronous.
            //      so allow some time for it to setup 'send' buffer.
            Alarm::sleep_for(Milliseconds(200)).unwrap();

            let r = I2CMasterSlave::i2c_master_slave_read_sync(addr, &mut rx_buf, rx_len);
            match r.1 {
                Ok(()) => {
                    writeln!(
                        Console::writer(),
                        "{:} bytes read from slave | data received (0h): {:x?}\r\n",
                        r.0,
                        rx_buf
                    )
                    .unwrap();
                }
                Err(why) => {
                    writeln!(
                        Console::writer(),
                        "i2c-master: read operation failed {:?}",
                        why
                    )
                    .unwrap();
                }
            }
            i += 1;
        }
        Alarm::sleep_for(Milliseconds(1000)).unwrap();
    }
}
