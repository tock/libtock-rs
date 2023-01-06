use std::cell::Cell;
use libtock_platform::{CommandReturn, ErrorCode};
use crate::{DriverInfo, DriverShareRef};

pub struct Temperature{
    busy: Cell<bool>,
    share_ref: DriverShareRef,
}

impl Temperature {
    pub fn new() -> std::rc::Rc<Temperature> {
        #[allow(clippy::declare_interior_mutable_const)]
        const NOT_BUSY: Cell<bool> = Cell::new(false);
        std::rc::Rc::new(Temperature {
            busy: NOT_BUSY,
            share_ref: Default::default(),
        })
    }
    
    pub fn is_busy(&self) -> bool{
        self.busy.get()
    }
    pub fn set_value(&self, value: i32) {
        if self.busy.get() {
            self.share_ref
                .schedule_upcall(0, (value as u32, 0, 0))
                .expect("Unable to schedule upcall");
            self.busy.set(false);
        }
    }
}

impl crate::fake::SyscallDriver for Temperature {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_id: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_id {
            EXISTS => crate::command_return::success(),

            READ_TEMP => {
                if !self.busy.get(){
                    self.busy.set(true);
                    crate::command_return::success()
                } 
                else {
                    crate::command_return::failure(ErrorCode::Busy)    
                }
            },
            _ => crate::command_return::failure(ErrorCode::NoSupport)
        }
    }
}

#[cfg(test)]
mod tests;
// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60000;

// Command IDs

const EXISTS: u32 = 0;
const READ_TEMP: u32 = 1;
