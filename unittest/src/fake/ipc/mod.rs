use libtock_platform::{CommandReturn, ErrorCode};

use crate::DriverInfo;

pub struct Ipc {}

impl Ipc {
    pub fn new() -> std::rc::Rc<Ipc> {
        std::rc::Rc::new(Ipc {})
    }
}

impl crate::fake::SyscallDriver for Ipc {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM)
    }

    fn command(&self, command_num: u32, argument0: u32, argument1: u32) -> CommandReturn {
        match command_num {
            EXISTS => crate::command_return::success(),
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x10000;

// Command IDs
const EXISTS: u32 = 0;
