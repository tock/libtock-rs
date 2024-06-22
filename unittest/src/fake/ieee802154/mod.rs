//! Fake implementation of the raw IEEE 802.15.4 API.

use core::cell::Cell;
use libtock_platform::{CommandReturn, ErrorCode};
use std::{
    cell::RefCell,
    collections::VecDeque,
    convert::TryFrom,
    rc::{self, Rc},
};

use crate::{command_return, DriverInfo, DriverShareRef, RoAllowBuffer, RwAllowBuffer};

/// Maximum length of a MAC frame.
const MAX_MTU: usize = 127;

const PSDU_OFFSET: usize = 2;

#[derive(Debug)]
#[repr(C)]
pub struct Frame {
    pub header_len: u8,
    pub payload_len: u8,
    pub mic_len: u8,
    pub body: [u8; MAX_MTU],
}

impl Frame {
    pub fn with_body(body: &[u8]) -> Self {
        let mut frame = Self {
            header_len: 0,
            payload_len: u8::try_from(body.len()).unwrap(),
            mic_len: 0,
            body: [0_u8; 127],
        };

        frame.body[PSDU_OFFSET..PSDU_OFFSET + body.len()].copy_from_slice(body);

        frame
    }
}

pub struct Ieee802154Phy {
    pan: Cell<u16>,
    addr_short: Cell<u16>,
    addr_long: Cell<u64>,
    chan: Cell<u8>,
    tx_power: Cell<i8>,
    radio_on: Cell<bool>,

    tx_buf: Cell<RoAllowBuffer>,
    rx_buf: RefCell<RwAllowBuffer>,

    transmitted_frames: Cell<Vec<Vec<u8>>>,

    frames_to_be_received: RefCell<VecDeque<Frame>>,

    share_ref: DriverShareRef,
}

// Needed for scheduling an receive upcall immediately after subscribing to it.
// Without that,

thread_local!(pub(crate) static DRIVER: RefCell<rc::Weak<Ieee802154Phy>> = const { RefCell::new(rc::Weak::new()) });

impl Ieee802154Phy {
    pub fn instance() -> Option<Rc<Self>> {
        DRIVER.with_borrow(|driver| driver.upgrade())
    }

    pub fn new() -> Rc<Self> {
        let new = Self::new_with_frames_to_be_received(std::iter::empty());
        DRIVER.with_borrow_mut(|inner| *inner = Rc::downgrade(&new));
        new
    }

    pub fn new_with_frames_to_be_received(
        frames_to_be_received: impl IntoIterator<Item = Frame>,
    ) -> Rc<Self> {
        Rc::new(Self {
            pan: Default::default(),
            addr_short: Default::default(),
            addr_long: Default::default(),
            chan: Default::default(),
            tx_power: Default::default(),
            radio_on: Default::default(),
            tx_buf: Default::default(),
            rx_buf: Default::default(),
            transmitted_frames: Default::default(),
            frames_to_be_received: RefCell::new(frames_to_be_received.into_iter().collect()),
            share_ref: Default::default(),
        })
    }

    pub fn take_transmitted_frames(&self) -> Vec<Vec<u8>> {
        self.transmitted_frames.take()
    }

    pub fn has_pending_rx_frames(&self) -> bool {
        let rx_buf = self.rx_buf.borrow();

        #[allow(clippy::get_first)]
        let read_index = rx_buf.get(0);
        let write_index = rx_buf.get(1);

        matches!((read_index, write_index), (Some(r), Some(w)) if r != w)
    }

    pub fn radio_receive_frame(&self, frame: Frame) {
        self.frames_to_be_received.borrow_mut().push_back(frame);
    }

    pub fn driver_receive_pending_frames(&self) {
        for frame in self.frames_to_be_received.borrow_mut().drain(..) {
            self.driver_receive_frame(&frame.body[..frame.payload_len as usize + PSDU_OFFSET]);
        }
    }

    fn driver_receive_frame(&self, frame: &[u8]) {
        let mut rx_buf = self.rx_buf.borrow_mut();
        Self::phy_driver_receive_frame(&mut rx_buf, frame);
    }

    // Code taken and adapted from capsules/extra/src/ieee802154/phy_driver.rs.
    fn phy_driver_receive_frame(rbuf: &mut [u8], frame: &[u8]) {
        let frame_len = frame.len() - PSDU_OFFSET;

        ////////////////////////////////////////////////////////
        // NOTE: context for the ring buffer and assumptions
        // regarding the ring buffer format and usage can be
        // found in the detailed comment at the top of this
        // file.
        //
        // Ring buffer format:
        //  | read  | write | user_frame | user_frame |...| user_frame |
        //  | index | index | 0          | 1          |   | n          |
        //
        // user_frame format:
        //  | header_len | payload_len | mic_len | 15.4 frame |
        //
        ////////////////////////////////////////////////////////

        const PSDU_OFFSET: usize = 2;

        // 2 bytes for the readwrite buffer metadata (read and
        // write index).
        const RING_BUF_METADATA_SIZE: usize = 2;

        /// 3 byte metadata (offset, len, mic_len)
        const USER_FRAME_METADATA_SIZE: usize = 3;

        /// 3 byte metadata + 127 byte max payload
        const USER_FRAME_MAX_SIZE: usize = USER_FRAME_METADATA_SIZE + MAX_MTU;

        // Confirm the availability of the buffer. A buffer of
        // len 0 is indicative of the userprocess not allocating
        // a readwrite buffer. We must also confirm that the
        // userprocess correctly formatted the buffer to be of
        // length 2 + n * USER_FRAME_MAX_SIZE, where n is the
        // number of user frames that the buffer can store. We
        // combine checking the buffer's non-zero length and the
        // case of the buffer being shorter than the
        // `RING_BUF_METADATA_SIZE` as an invalid buffer (e.g.
        // of length 1) may otherwise errantly pass the second
        // conditional check (due to unsigned integer
        // arithmetic).
        assert!(rbuf.len() > RING_BUF_METADATA_SIZE);
        assert!((rbuf.len() - RING_BUF_METADATA_SIZE) % USER_FRAME_MAX_SIZE == 0);

        let mut read_index = rbuf[0] as usize;
        let mut write_index = rbuf[1] as usize;

        let max_pending_rx = (rbuf.len() - RING_BUF_METADATA_SIZE) / USER_FRAME_MAX_SIZE;

        // Confirm user modifiable metadata is valid (i.e.
        // within bounds of the provided buffer).
        assert!(read_index < max_pending_rx && write_index < max_pending_rx);

        // We don't parse the received packet, so we don't know
        // how long all of the pieces are.
        let mic_len = 0;
        let header_len = 0;

        // Start in the buffer where we are going to write this
        // incoming packet.
        let offset = RING_BUF_METADATA_SIZE + (write_index * USER_FRAME_MAX_SIZE);

        // Copy the entire frame over to userland, preceded by
        // three metadata bytes: the header length, the data
        // length, and the MIC length.
        let dst_start = offset + USER_FRAME_METADATA_SIZE;
        let dst_end = dst_start + frame_len;
        let src_start = PSDU_OFFSET;
        let src_end = src_start + frame_len;
        rbuf[dst_start..dst_end].copy_from_slice(&frame[src_start..src_end]);

        rbuf[offset] = header_len as u8;
        rbuf[offset + 1] = frame_len as u8;
        rbuf[offset + 2] = mic_len as u8;

        // Prepare the ring buffer for the next write. The
        // current design favors newness; newly received packets
        // will begin to overwrite the oldest data in the event
        // of the buffer becoming full. The read index must
        // always point to the "oldest" data. If we have
        // overwritten the oldest data, the next oldest data is
        // now at the read index + 1. We must update the read
        // index to reflect this.
        write_index = (write_index + 1) % max_pending_rx;
        if write_index == read_index {
            read_index = (read_index + 1) % max_pending_rx;
            rbuf[0] = read_index as u8;
        }

        // Update write index metadata since we have added a
        // frame.
        rbuf[1] = write_index as u8;
    }

    pub fn trigger_rx_upcall(&self) {
        self.share_ref
            .schedule_upcall(subscribe::FRAME_RECEIVED, (0, 0, 0))
            .expect("Unable to schedule upcall {}");
    }
}

impl crate::fake::SyscallDriver for Ieee802154Phy {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(2)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_number: u32, argument0: u32, argument1: u32) -> CommandReturn {
        match command_number {
            command::EXISTS => command_return::success(),
            command::STATUS => {
                if self.radio_on.get() {
                    command_return::success()
                } else {
                    command_return::failure(ErrorCode::Off)
                }
            }
            command::SET_SHORT_ADDR => {
                self.addr_short.set(u16::try_from(argument0).unwrap());
                command_return::success()
            }
            command::SET_PAN => {
                self.pan.set(u16::try_from(argument0).unwrap());
                command_return::success()
            }
            command::SET_CHAN => {
                self.chan.set(u8::try_from(argument0).unwrap());
                command_return::success()
            }
            command::SET_TX_PWR => {
                self.tx_power.set(i8::try_from(argument0 as i32).unwrap());
                command_return::success()
            }
            command::COMMIT_CFG => command_return::success(),
            command::GET_SHORT_ADDR => command_return::success_u32(self.addr_short.get() as u32),
            command::GET_PAN => command_return::success_u32(self.pan.get() as u32),
            command::GET_CHAN => command_return::success_u32(self.chan.get() as u32),
            command::GET_TX_PWR => command_return::success_u32(self.tx_power.get() as i32 as u32),
            command::SET_LONG_ADDR => {
                self.addr_long
                    .set(argument0 as u64 | (argument1 as u64) << 32);
                command_return::success()
            }
            command::GET_LONG_ADDR => command_return::success_u64(self.addr_long.get()),
            command::TURN_ON => {
                self.radio_on.set(true);
                command_return::success()
            }
            command::TURN_OFF => {
                self.radio_on.set(false);
                command_return::success()
            }
            command::TRANSMIT => {
                let mut transmitted_frames = self.transmitted_frames.take();
                let tx_buf = self.tx_buf.take();
                transmitted_frames.push(Vec::from(tx_buf.as_ref()));

                self.tx_buf.set(tx_buf);
                self.transmitted_frames.set(transmitted_frames);
                self.share_ref
                    .schedule_upcall(subscribe::FRAME_TRANSMITTED, (2137, 0, 0))
                    .expect("Unable to schedule upcall {}");

                command_return::success()
            }
            _ => command_return::failure(ErrorCode::Invalid),
        }
    }

    fn allow_readonly(
        &self,
        buffer_num: u32,
        buffer: crate::RoAllowBuffer,
    ) -> Result<crate::RoAllowBuffer, (crate::RoAllowBuffer, ErrorCode)> {
        if buffer_num == allow_ro::WRITE {
            Ok(self.tx_buf.replace(buffer))
        } else {
            Err((buffer, ErrorCode::Invalid))
        }
    }

    fn allow_readwrite(
        &self,
        buffer_num: u32,
        buffer: crate::RwAllowBuffer,
    ) -> Result<crate::RwAllowBuffer, (crate::RwAllowBuffer, ErrorCode)> {
        if buffer_num == allow_rw::READ {
            Ok(self.rx_buf.replace(buffer))
        } else {
            Err((buffer, ErrorCode::Invalid))
        }
    }
}

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

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
