use libtock_platform::ErrorCode;

use crate::{DriverInfo, DriverShareRef, RwAllowBuffer};
use std::cell::{Cell, RefCell};
use std::ops::DerefMut;

pub struct Rng {
    busy: Cell<bool>,
    share_ref: DriverShareRef,
    buffer: RefCell<RwAllowBuffer>,
    bytes: Cell<Vec<u8>>,
}

impl Rng {
    pub fn new() -> std::rc::Rc<Rng> {
        std::rc::Rc::new(Rng {
            busy: Cell::new(false),
            share_ref: Default::default(),
            buffer: Default::default(),
            bytes: Default::default(),
        })
    }

    pub fn is_busy(&self) -> bool {
        self.busy.get()
    }
}

impl crate::fake::SyscallDriver for Rng {
    fn info(&self) -> crate::DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_id: u32, argument0: u32, _: u32) -> libtock_platform::CommandReturn {
        match command_id {
            EXISTS => crate::command_return::success(),
            GET_BYTES => {
                if self.busy.get() {
                    return crate::command_return::failure(ErrorCode::Busy);
                }
                self.busy.set(true);
                // Mutably borrows RwAllowBuffer from the RefCell as to not take the buffer out
                let mut allow_rw_buffer = self.buffer.borrow_mut();
                let inner_buffer = (*allow_rw_buffer).deref_mut();
                inner_buffer.copy_from_slice(
                    &self.bytes.take()[0..core::cmp::min(inner_buffer.len(), argument0 as usize)],
                );
                crate::command_return::success()
            }
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }

    fn allow_readwrite(
        &self,
        buffer_num: u32,
        buffer: crate::RwAllowBuffer,
    ) -> Result<crate::RwAllowBuffer, (crate::RwAllowBuffer, libtock_platform::ErrorCode)> {
        if buffer_num == 0 {
            Ok(self.buffer.replace(buffer))
        } else {
            Err((buffer, ErrorCode::Invalid))
        }
    }
}

#[cfg(test)]
mod tests;

// -----------------
// RNG DRIVER NUMBER
// -----------------
const DRIVER_NUM: u32 = 0x40001;

// -------------------
// RNG COMMAND NUMBERS
// -------------------
const EXISTS: u32 = 0;
const GET_BYTES: u32 = 1;
