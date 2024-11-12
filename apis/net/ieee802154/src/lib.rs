//! The raw IEEE 802.15.4 stack driver.

#![no_std]

use core::cell::Cell;
use libtock_platform as platform;
use libtock_platform::allow_ro::AllowRo;
use libtock_platform::allow_rw::AllowRw;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

/// The raw IEEE 802.15.4 stack driver.
///
/// It allows libraries to pass frames to and from kernel's 802.15.4 driver.
///
/// # Example
/// ```ignore
/// use libtock::ieee802154::{Ieee802154, RxOperator, RxRingBuffer, RxSingleBufferOperator};
///
/// // Configure the radio
/// let pan: u16 = 0xcafe;
/// let addr_short: u16 = 0xdead;
/// let addr_long: u64 = 0xdead_dad;
/// let tx_power: i8 = -0x42;
/// let channel: u8 = 0xff;
///
/// Ieee802154::set_pan(pan);
/// Ieee802154::set_address_short(addr_short);
/// Ieee802154::set_address_long(addr_long);
/// Ieee802154::set_tx_power(tx_power).unwrap();
/// Ieee802154::set_channel(channel).unwrap();
///
/// // Don't forget to commit the config!
/// Ieee802154::commit_config();
///
/// Ieee802154::radio_on()?;
///
/// // Transmit a frame
/// Ieee802154::transmit_frame(b"foobar").unwrap();
///
/// // Receive frames
/// let mut buf = RxRingBuffer::<2>::new();
/// let mut operator = RxSingleBufferOperator::new(&mut buf);
///
/// let frame = operator.receive_frame()?;
/// // Access frame data here:
/// let _body_len = frame.payload_len;
/// let _first_body_byte = frame.body[0];
///
/// ```
pub struct Ieee802154<S: Syscalls, C: Config = DefaultConfig>(S, C);

// Existence check
impl<S: Syscalls, C: Config> Ieee802154<S, C> {
    /// Run a check against the console capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn exists() -> bool {
        S::command(DRIVER_NUM, command::EXISTS, 0, 0).is_success()
    }
}

// Power management
impl<S: Syscalls, C: Config> Ieee802154<S, C> {
    #[inline(always)]
    pub fn is_on() -> bool {
        S::command(DRIVER_NUM, command::STATUS, 0, 0).is_success()
    }

    #[inline(always)]
    pub fn radio_on() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::TURN_ON, 0, 0).to_result()
    }

    #[inline(always)]
    pub fn radio_off() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::TURN_OFF, 0, 0).to_result()
    }
}

// Configuration
impl<S: Syscalls, C: Config> Ieee802154<S, C> {
    #[inline(always)]
    pub fn set_address_short(short_addr: u16) {
        // Setting short address can't fail, so no need to check the return value.
        let _ = S::command(
            DRIVER_NUM,
            command::SET_SHORT_ADDR,
            // Driver expects 1 added to make the value positive.
            short_addr as u32 + 1,
            0,
        );
    }

    #[inline(always)]
    pub fn set_address_long(long_addr: u64) {
        // Setting long address can't fail, so no need to check the return value.
        let addr_lower: u32 = long_addr as u32;
        let addr_upper: u32 = (long_addr >> 32) as u32;
        let _ = S::command(DRIVER_NUM, command::SET_LONG_ADDR, addr_lower, addr_upper);
    }

    #[inline(always)]
    pub fn set_pan(pan: u16) {
        // Setting PAN can't fail, so no need to check the return value.
        let _ = S::command(
            DRIVER_NUM,
            command::SET_PAN,
            pan as u32 + 1, // Driver expects 1 added to make the value positive.
            0,
        );
    }

    #[inline(always)]
    pub fn set_channel(chan: u8) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::SET_CHAN, chan as u32, 0).to_result()
    }

    #[inline(always)]
    pub fn set_tx_power(power: i8) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::SET_TX_PWR, power as i32 as u32, 0).to_result()
    }

    #[inline(always)]
    pub fn commit_config() {
        // Committing config can't fail, so no need to check the return value.
        let _ = S::command(DRIVER_NUM, command::COMMIT_CFG, 0, 0);
    }

    #[inline(always)]
    pub fn get_address_short() -> Result<u16, ErrorCode> {
        S::command(DRIVER_NUM, command::GET_SHORT_ADDR, 0, 0)
            .to_result::<u32, _>()
            // Driver adds 1 to make the value positive.
            .map(|addr| addr as u16 - 1)
    }

    #[inline(always)]
    pub fn get_address_long() -> Result<u64, ErrorCode> {
        S::command(DRIVER_NUM, command::GET_LONG_ADDR, 0, 0).to_result()
    }

    #[inline(always)]
    pub fn get_pan() -> Result<u16, ErrorCode> {
        S::command(DRIVER_NUM, command::GET_PAN, 0, 0)
            .to_result::<u32, _>()
            // Driver adds 1 to make the value positive.
            .map(|pan| pan as u16 - 1)
    }

    #[inline(always)]
    pub fn get_channel() -> Result<u8, ErrorCode> {
        S::command(DRIVER_NUM, command::GET_CHAN, 0, 0)
            .to_result::<u32, _>()
            .map(|chan| chan as u8)
    }

    #[inline(always)]
    pub fn get_tx_power() -> Result<i8, ErrorCode> {
        S::command(DRIVER_NUM, command::GET_TX_PWR, 0, 0)
            .to_result::<u32, _>()
            .map(|power| power as i32 as i8)
    }
}

// Transmission
impl<S: Syscalls, C: Config> Ieee802154<S, C> {
    pub fn transmit_frame(frame: &[u8]) -> Result<(), ErrorCode> {
        let called: Cell<Option<(u32,)>> = Cell::new(None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { allow_ro::WRITE }>,
                Subscribe<_, DRIVER_NUM, { subscribe::FRAME_TRANSMITTED }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();

            S::allow_ro::<C, DRIVER_NUM, { allow_ro::WRITE }>(allow_ro, frame)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::FRAME_TRANSMITTED }>(
                subscribe, &called,
            )?;

            S::command(DRIVER_NUM, command::TRANSMIT, 0, 0).to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if called.get().is_some() {
                    return Ok(());
                }
            }
        })
    }
}

mod rx;
pub use rx::{Frame, RxOperator, RxRingBuffer, RxSingleBufferOperator};

/// System call configuration trait for `Ieee802154`.
pub trait Config:
    platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config
{
}
impl<T: platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config>
    Config for T
{
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x30001;

// Command IDs
/// - `0`: Driver existence check.
/// - `1`: Return radio status. Ok(())/OFF = on/off.
/// - `2`: Set short address.
/// - `4`: Set PAN ID.
/// - `5`: Set channel.
/// - `6`: Set transmission power.
/// - `7`: Commit any configuration changes.
/// - `8`: Get the short MAC address.
/// - `10`: Get the PAN ID.
/// - `11`: Get the channel.
/// - `12`: Get the transmission power.
/// - `27`: Transmit a frame. The frame must be stored in the write RO allow
///   buffer 0. The allowed buffer must be the length of the frame. The
///   frame includes the PDSU (i.e., the MAC payload) _without_ the MFR
///   (i.e., CRC) bytes.
/// - `28`: Set long address.
/// - `29`: Get the long MAC address.
/// - `30`: Turn the radio on.
/// - `31`: Turn the radio off.
mod command {
    pub const EXISTS: u32 = 0;
    pub const STATUS: u32 = 1;
    pub const SET_SHORT_ADDR: u32 = 2;
    pub const SET_PAN: u32 = 4;
    pub const SET_CHAN: u32 = 5;
    pub const SET_TX_PWR: u32 = 6;
    pub const COMMIT_CFG: u32 = 7;
    pub const GET_SHORT_ADDR: u32 = 8;
    pub const GET_PAN: u32 = 10;
    pub const GET_CHAN: u32 = 11;
    pub const GET_TX_PWR: u32 = 12;
    pub const TRANSMIT: u32 = 27;
    pub const SET_LONG_ADDR: u32 = 28;
    pub const GET_LONG_ADDR: u32 = 29;
    pub const TURN_ON: u32 = 30;
    pub const TURN_OFF: u32 = 31;
}

mod subscribe {
    /// Frame is received
    pub const FRAME_RECEIVED: u32 = 0;
    /// Frame is transmitted
    pub const FRAME_TRANSMITTED: u32 = 1;
}

/// Ids for read-only allow buffers
mod allow_ro {
    /// Write buffer. Contains the frame payload to be transmitted.
    pub const WRITE: u32 = 0;
}

/// Ids for read-write allow buffers
mod allow_rw {
    /// Read buffer. Will contain the received frame.
    pub const READ: u32 = 0;
}
