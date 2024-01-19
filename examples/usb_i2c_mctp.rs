//! A sample app that implements MCTP messages to be transceived from a host
//! machine without exposed SMBus/I2C capabilities to a target endpoint using
//! UART and I2C.
//!
//! The following topology is used:
//!
//! [HOST MACHINE] <--UART--> [USB_I2C_BRIDGE_DEVICE] <--I2C/SmBus--> [TARGET_ENDPOiNT]
//!
//! The host machine will issue a message to the USB_I2C_BRIDGE_DEVICE, which runs this app.
//! The app then determines the end point target address based on the received packet header,
//! and forwards this data to the target endpoint. As we are supporting MCTP, there are no
//! I2C reads, only write. Thus, the endpoint must then master the bus and write a response
//! back. This message is forwarded to the host again via UART.
//!
//! The host application must append a small packet header of the following format
//! for any messages being send to this device.
//!
//! host_tx[0] = 0xAA // Preamble
//! host_tx[1] = XX   // Endpoint target address
//! host_tx[2] = YY   // MSB of 16bit data length
//! host_tx[3] = ZZ   // LSB of 16bit data length
//!
//! For reception, the device (this app) first sends a packet header of the following format
//!
//! device_tx[0] = 0xBB // Preamble
//! device_tx[1] = 0xFF // Unused
//! device_tx[2] = YY   // MSB of 16bit data length
//! device_tx[3] = ZZ   // LSB of 16bit data length
//!
//! Based on the data length, the host can read the next ((YY << 8) | ZZ )
//! as data message.
//!
//! Required Kernel Configuration:
//!
//! This application requires that the kernel console buffer sizes are increased
//! as well as i2c-master-slave driver buffers. This is because we are doing transfers
//! of sizes greater than what the upstream kernel is allowed to do.
//!
//! The following reference can be used to prepare the kernel for the uart/console buffers.
//! The size specified here must be >= RX_BUF_LEN (as specified below).
//!
//! ```
//! diff --git a/capsules/core/src/console.rs b/capsules/core/src/console.rs
//! index a3b3af6cf..4eaf401a9 100644
//! --- a/capsules/core/src/console.rs
//! +++ b/capsules/core/src/console.rs
//! @@ -54,7 +54,7 @@ pub const DRIVER_NUM: usize = driver::NUM::Console as usize;
//!
//!  /// Default size for the read and write buffers used by the console.
//!  /// Boards may pass different-size buffers if needed.
//! -pub const DEFAULT_BUF_SIZE: usize = 64;
//! +pub const DEFAULT_BUF_SIZE: usize = 132;
//!
//!  /// IDs for subscribed upcalls.
//!  mod upcall {
//! ```
//!
//! ```
//! diff --git a/capsules/core/src/virtualizers/virtual_uart.rs b/capsules/core/src/virtualizers/virtual_uart.rs
//! index 0d39fe024..59e0435a1 100644
//! --- a/capsules/core/src/virtualizers/virtual_uart.rs
//! +++ b/capsules/core/src/virtualizers/virtual_uart.rs
//! @@ -54,7 +54,7 @@ use kernel::hil::uart;
//!  use kernel::utilities::cells::{OptionalCell, TakeCell};
//!  use kernel::ErrorCode;
//!
//! -pub const RX_BUF_LEN: usize = 64;
//! +pub const RX_BUF_LEN: usize = 132;
//!
//!  pub struct MuxUart<'a> {
//!      uart: &'a dyn uart::Uart<'a>,
//! ```
//!
//! For i2c-master-slave buffers, use a buffer size that is >= MAX_DLEN
//! (as specified below).
//!

#![no_main]
#![no_std]
use libtock::console::Console;
use libtock::i2c_master_slave::I2CMasterSlave;
use libtock::leds::Leds;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x900}

/// The address to which we listen for in slave/target mode.
pub const MY_ID: u8 = 0x34;
/// Contains packet metadata
pub const HEADER_LEN: usize = 4;
/// Max data message length
pub const MAX_DLEN: usize = 128;
/// Total amount of bytes we can receive in a single UART RX
pub const RX_BUF_LEN: usize = HEADER_LEN + MAX_DLEN;
/// Debug LED config, change these based on the board config
pub const PANIC_LED: u32 = 0;
/// Triggered when waiting for RX
pub const RX_LED: u32 = 1;
/// Triggered when TX in progress
pub const TX_LED: u32 = 2;

/// # Summary
///
/// A helper function to append the packet metadata do the outgoing buffer
/// pointed to by @buf.
///
/// # Parameter
///
/// * `buf`: the buffer in which to append the metadata to.
/// * `n_bytes`: length of data message body of the next message this is
///              stored in the packet header, so that the host can
///              determine the data message read length.
///
/// # Returns
///
/// Ok(n) returns the total size of bytes to send from this buffer (buf[0..n]).
///
/// # Panics
///
/// If the buffer capacity cannot fit the packet metadata
///
fn prepare_tx_header(buf: &mut [u8], n_bytes: usize) -> Result<usize, ()> {
    let total_bytes_to_send = HEADER_LEN;
    if total_bytes_to_send > buf.len() {
        return Err(());
    }
    // Setup Header
    buf[0] = 0xBB;
    buf[1] = 0xFF;
    // Upper 8-bits
    buf[2..=3].copy_from_slice(&u16::to_be_bytes(n_bytes as u16));

    Ok(total_bytes_to_send)
}

fn main() {
    let led_count = Leds::count().unwrap_or(0);

    // Using the led number, set it on iff it's available
    let led_on = |led_num| {
        if led_num < led_count {
            Leds::on(led_num).unwrap()
        }
    };

    // Using the led number, set it off iff it's available
    let led_off = |led_num| {
        if led_num < led_count {
            Leds::off(led_num).unwrap()
        }
    };

    // RX Buffer layout
    // [0] = Preamble
    // [1] = TargetID
    // [2] = Length Upper Byte
    // [3] = Length Lower Byte
    let mut rx_buf: [u8; RX_BUF_LEN] = [0x00; RX_BUF_LEN];
    let mut msg_len: u16;
    let mut target_id: u16;

    loop {
        led_on(RX_LED);
        let (_, err) = Console::read(&mut rx_buf);
        led_off(RX_LED);

        if err.is_err() {
            led_on(PANIC_LED);
            panic!("Failed to read from host {:?}", err);
        }
        // If we don't get a matching preamble, then the rest of the data is unreliable.
        assert_eq!(rx_buf[0], 0xAA);
        // Target in 7-bit address range?
        assert!(rx_buf[1] <= 0x7F);
        target_id = rx_buf[1] as u16;
        // Data length should be non-zero, otherwise why are we here? just to suffer?
        msg_len = u16::from_be_bytes([rx_buf[2], rx_buf[3]]);
        assert!(msg_len as usize <= MAX_DLEN);
        assert_ne!(msg_len, 0);

        if let Err(why) = I2CMasterSlave::i2c_master_slave_write_sync(
            target_id as u16,
            &mut rx_buf[HEADER_LEN..HEADER_LEN + msg_len as usize],
            msg_len as u16,
        ) {
            led_on(PANIC_LED);
            panic!("i2c-master: write operation failed {:?}", why);
        }

        I2CMasterSlave::i2c_master_slave_set_slave_address(MY_ID)
            .expect("i2c-target: Failed to set slave address");

        // Expect a write, if the master reads here, the IP may stretch clocks!
        let r = I2CMasterSlave::i2c_master_slave_write_recv_sync(&mut rx_buf[HEADER_LEN..]);

        if let Err(why) = r.1 {
            led_on(PANIC_LED);
            panic!("i2c-slave: error to receiving data {:?}\r", why);
        }
        let mut header: [u8; HEADER_LEN] = [0; HEADER_LEN];
        let mut tx_len = 0;
        if let Ok(n) = prepare_tx_header(&mut header, r.0) {
            tx_len = n;
        }
        assert_eq!(tx_len, HEADER_LEN);

        // Write header first, this allows the host to know how many bytes to
        // expect in the following data message.
        led_on(TX_LED);
        if Console::write(&mut header).is_err() {
            led_on(PANIC_LED);
        }

        // Data message body.
        if Console::write(&mut rx_buf[HEADER_LEN..HEADER_LEN + r.0]).is_err() {
            led_on(PANIC_LED);
        }
        led_off(TX_LED);
    }
}
