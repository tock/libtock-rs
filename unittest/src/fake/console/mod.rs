//! Fake implementation of the Console API, documented here:
//! https://github.com/tock/tock/blob/master/doc/syscalls/00001_console.md
//!
//! Like the real API, `Console` stores each message written to it.
//! The resulting byte stream can be retrieved via `take_bytes`
//! for use in unit tests.

use core::cell::{Cell, RefCell};
use core::cmp;
use libtock_platform::{CommandReturn, ErrorCode};

use crate::{DriverInfo, DriverShareRef, RoAllowBuffer, RwAllowBuffer};

pub struct Console {
    messages: Cell<Vec<u8>>,
    buffer: Cell<RoAllowBuffer>,

    read_buffer: RefCell<RwAllowBuffer>,
    /// To be returned on read
    input: Cell<Vec<u8>>,

    share_ref: DriverShareRef,
}

impl Console {
    pub fn new() -> std::rc::Rc<Console> {
        Self::new_with_input(b"")
    }

    pub fn new_with_input(inputs: &[u8]) -> std::rc::Rc<Console> {
        std::rc::Rc::new(Console {
            messages: Default::default(),
            buffer: Default::default(),
            read_buffer: Default::default(),
            input: Cell::new(Vec::from(inputs)),
            share_ref: Default::default(),
        })
    }

    /// Returns the bytes that have been submitted so far,
    /// and clears them.
    pub fn take_bytes(&self) -> Vec<u8> {
        self.messages.take()
    }
}

impl crate::fake::SyscallDriver for Console {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(3)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn allow_readonly(
        &self,
        buffer_num: u32,
        buffer: RoAllowBuffer,
    ) -> Result<RoAllowBuffer, (RoAllowBuffer, ErrorCode)> {
        if buffer_num == ALLOW_WRITE {
            Ok(self.buffer.replace(buffer))
        } else {
            Err((buffer, ErrorCode::Invalid))
        }
    }

    fn allow_readwrite(
        &self,
        buffer_num: u32,
        buffer: RwAllowBuffer,
    ) -> Result<RwAllowBuffer, (RwAllowBuffer, ErrorCode)> {
        if buffer_num == ALLOW_READ {
            Ok(self.read_buffer.replace(buffer))
        } else {
            Err((buffer, ErrorCode::Invalid))
        }
    }

    fn command(&self, command_num: u32, argument0: u32, _argument1: u32) -> CommandReturn {
        match command_num {
            EXISTS => {}
            WRITE => {
                let mut bytes = self.messages.take();
                let buffer = self.buffer.take();
                let size = cmp::min(buffer.len(), argument0 as usize);
                bytes.extend_from_slice(&(*buffer)[..size]);
                self.buffer.set(buffer);
                self.messages.set(bytes);
                self.share_ref
                    .schedule_upcall(SUBSCRIBE_WRITE, (size as u32, 0, 0))
                    .expect("Unable to schedule upcall {}");
            }
            READ => {
                let count_wanted = argument0 as usize;
                let bytes = self.input.take();
                let count_wanted = cmp::min(count_wanted, bytes.len());
                let to_send = &bytes[..count_wanted];
                let to_keep = &bytes[count_wanted..];
                self.input.set(Vec::from(to_keep));

                let count_available = to_send.len();
                self.read_buffer.borrow_mut()[..count_wanted].copy_from_slice(to_send);
                self.share_ref
                    .schedule_upcall(SUBSCRIBE_READ, (0, count_available as u32, 0))
                    .expect("Unable to schedule upcall {}");
            }
            _ => return crate::command_return::failure(ErrorCode::NoSupport),
        }
        crate::command_return::success()
    }
}

// -----------------------------------------------------------------------------
// Implementation details below
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests;

const DRIVER_NUM: u32 = 0x1;

// Command numbers
const EXISTS: u32 = 0;
const WRITE: u32 = 1;
const READ: u32 = 2;
//const ABORT: u32 = 3;
const SUBSCRIBE_WRITE: u32 = 1;
const SUBSCRIBE_READ: u32 = 2;
const ALLOW_WRITE: u32 = 1;
const ALLOW_READ: u32 = 1;
