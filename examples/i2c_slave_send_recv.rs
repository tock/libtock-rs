//! This sample demonstrates setting up the i2c ip (assuming board has support)
//! for target mode. In the event loop, we first expect the master to write some data
//! then we setup a response packet.
//!
//! NOTE: The device (based on hwip) may stretch clocks by holding the SCL line low if the master attempts to
//! read data before we have setup the read data buffers.
//!
//! This sample is tested with `i2c_master_write_read.rs` sample running on the
//! master device.

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::console::Console;
use libtock::i2c_master_slave::I2CMasterSlave;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x400}
pub const SLAVE_DEVICE_ADDR: u8 = 0x69;
fn main() {
    let mut rx_buf: [u8; 8] = [0; 8];
    let mut tx_buf: [u8; 8] = [0; 8];
    let addr: u8 = SLAVE_DEVICE_ADDR;
    // 7-bit addressing
    assert!(addr <= 0x7f);

    writeln!(Console::writer(), "i2c-slave: setting up\r").unwrap();
    writeln!(Console::writer(), "i2c-slave: address 0x{:x}!\r", addr).unwrap();

    I2CMasterSlave::i2c_master_slave_set_slave_address(addr).expect("i2c-target: Failed to listen");
    let mut i: u32 = 0;
    loop {
        writeln!(Console::writer(), "i2c-slave: operation {:?}\r", i).unwrap();

        // Expect a write, if the master reads here, the IP may stretch clocks!
        let r = I2CMasterSlave::i2c_master_slave_write_recv_sync(&mut rx_buf);

        if let Err(why) = r.1 {
            writeln!(
                Console::writer(),
                "i2c-slave: error to receiving data {:?}\r",
                why
            )
            .unwrap();
        } else {
            writeln!(
                Console::writer(),
                "{:} bytes received from master | buf: {:x?}\r",
                r.0,
                rx_buf
            )
            .unwrap();

            // Note: The master should allow a little delay when communicating with this slave
            //       as we are doing everything synchronously.
            // Expect a 2 byte read by master and let's keep changing the values
            tx_buf[0] = tx_buf[0].wrapping_add(1);
            tx_buf[1] = tx_buf[1].wrapping_add(5);
            let r = I2CMasterSlave::i2c_master_slave_read_send_sync(&tx_buf, tx_buf.len());

            match r.1 {
                Ok(()) => {
                    writeln!(
                        Console::writer(),
                        "{:} bytes read by master | data sent: {:x?}\r",
                        r.0,
                        tx_buf
                    )
                    .unwrap();
                    i += 1;
                }
                Err(why) => {
                    writeln!(
                        Console::writer(),
                        "i2c-slave: error setting up read_send {:?}\r",
                        why
                    )
                    .unwrap();
                }
            }
        }
    }
}
