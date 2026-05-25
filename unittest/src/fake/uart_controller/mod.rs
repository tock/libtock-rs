use core::cell::{Cell, RefCell};
use core::cmp;
use libtock_platform::{CommandReturn, ErrorCode};

use crate::{DriverInfo, DriverShareRef, RoAllowBuffer, RwAllowBuffer};

pub struct UartController {
    write_buffer: Cell<RoAllowBuffer>,
    read_buffer: RefCell<RwAllowBuffer>,
    last_write: Cell<Vec<u8>>,
    input: Cell<Vec<u8>>,
    share_ref: DriverShareRef,
}
impl UartController {
    pub fn new() -> std::rc::Rc<Self> {
        std::rc::Rc::new(Self {
            write_buffer: Default::default(),
            read_buffer: Default::default(),
            last_write: Default::default(),
            input: Default::default(),
            share_ref: Default::default(),
        })
    }
    pub fn set_read_data(&self, data: &[u8]) {
        self.input.set(data.to_vec());
    }
    pub fn get_last_write(&self) -> Vec<u8> {
        self.last_write.take()
    }
}
impl crate::fake::SyscallDriver for UartController {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
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
            Ok(self.write_buffer.replace(buffer))
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
    fn command(&self, command_id: u32, arg0: u32, _arg1: u32) -> CommandReturn {
        match command_id {
            EXISTS => crate::command_return::success_u32(1),
            WRITE => {
                let buffer = self.write_buffer.take();
                let size = cmp::min(buffer.len(), arg0 as usize);
                self.last_write.set(buffer[..size].to_vec());
                self.write_buffer.set(buffer);
                self.share_ref
                    .schedule_upcall(0, (size as u32, 0, 0))
                    .unwrap();
                crate::command_return::success()
            }
            READ => {
                let mut inb = self.input.take();
                let size = cmp::min(inb.len(), arg0 as usize);
                self.read_buffer.borrow_mut()[..size].copy_from_slice(&inb[..size]);
                inb.drain(..size);
                self.input.set(inb);
                self.share_ref
                    .schedule_upcall(0, (size as u32, 0, 0))
                    .unwrap();
                crate::command_return::success()
            }
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }
}

const DRIVER_NUM: u32 = 0x22;
const EXISTS: u32 = 0;
const WRITE: u32 = 1;
const READ: u32 = 2;
const ALLOW_WRITE: u32 = 1;
const ALLOW_READ: u32 = 1;
