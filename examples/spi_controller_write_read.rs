//! This sample demonstrates setting up the SPI controller (assuming board has support)

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size};
use libtock::spi_controller::SpiController;

set_main! {main}
stack_size! {0x400}

const OPERATION_LEN: usize = 0x08;

fn main() {
    let tx_buf: [u8; OPERATION_LEN] = [0x12; OPERATION_LEN];
    let mut rx_buf: [u8; OPERATION_LEN] = [0; OPERATION_LEN];

    writeln!(Console::writer(), "spi-controller: write-read\r").unwrap();
    if let Err(why) =
        SpiController::spi_controller_write_read_sync(&tx_buf, &mut rx_buf, OPERATION_LEN as u32)
    {
        writeln!(
            Console::writer(),
            "spi-controller: write-read operation failed {:?}\r",
            why
        )
        .unwrap();
    } else {
        writeln!(
            Console::writer(),
            "spi-controller: write-read: wrote {:x?}: read {:x?}\r",
            tx_buf,
            rx_buf
        )
        .unwrap();
    }

    writeln!(Console::writer(), "spi-controller: write\r").unwrap();
    if let Err(why) = SpiController::spi_controller_write_sync(&tx_buf, OPERATION_LEN as u32) {
        writeln!(
            Console::writer(),
            "spi-controller: write operation failed {:?}\r",
            why
        )
        .unwrap();
    } else {
        writeln!(Console::writer(), "spi-controller: wrote {:x?}\r", tx_buf).unwrap();
    }

    writeln!(Console::writer(), "spi-controller: read\r").unwrap();
    if let Err(why) = SpiController::spi_controller_read_sync(&mut rx_buf, OPERATION_LEN as u32) {
        writeln!(
            Console::writer(),
            "spi-controller: read operation failed {:?}\r",
            why
        )
        .unwrap();
    } else {
        writeln!(Console::writer(), "spi-controller: read {:x?}\r", rx_buf).unwrap();
    }

    writeln!(Console::writer(), "spi-controller: inplace write-read\r").unwrap();
    if let Err(why) =
        SpiController::spi_controller_inplace_write_read_sync(&mut rx_buf, OPERATION_LEN as u32)
    {
        writeln!(
            Console::writer(),
            "spi-controller: inplace write-read operation failed {:?}\r",
            why
        )
        .unwrap();
    } else {
        writeln!(
            Console::writer(),
            "spi-controller: inplace write-read: wrote {:x?}: read {:x?}\r",
            tx_buf,
            rx_buf
        )
        .unwrap();
    }
}
