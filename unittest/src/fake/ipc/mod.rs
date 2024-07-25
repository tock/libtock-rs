//! Fake implementation of the Ipc API, documented here:

use libtock_platform::{CommandReturn, ErrorCode};
use std::cell::RefCell;

use crate::{DriverInfo, DriverShareRef, RoAllowBuffer};

// TODO: remove identifier, just have processes be package names
// TODO: remove index, use .enumerate() on processes
// TODO: figure out how to simulate calls on processes

#[derive(Clone, Debug)]
pub struct Process {
    pkg_name: Vec<u8>,
}

impl Process {
    pub fn new(pkg_name: &[u8]) -> Process {
        Process {
            pkg_name: Vec::from(pkg_name),
        }
    }
}

pub struct Ipc {
    processes: Vec<Process>,
    search_buffer: RefCell<RoAllowBuffer>,
    share_ref: DriverShareRef,
}
impl Ipc {
    pub fn new() -> std::rc::Rc<Ipc> {
        Self::new_with_processes(&[] as &[&[u8]])
    }

    pub fn new_with_processes<T: AsRef<[u8]>>(pkg_names: &[T]) -> std::rc::Rc<Ipc> {
        std::rc::Rc::new(Ipc {
            processes: pkg_names
                .iter()
                .map(|name| Process::new(name.as_ref()))
                .collect(),
            search_buffer: Default::default(),
            share_ref: Default::default(),
        })
    }
}

impl crate::fake::SyscallDriver for Ipc {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM)
    }

    fn command(&self, command_num: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_num {
            command::EXISTS => crate::command_return::success(),
            command::DISCOVER => self
                .processes
                .iter()
                .position(|process| {
                    let search = self.search_buffer.borrow();
                    process.pkg_name.len() == search.len()
                        && process
                            .pkg_name
                            .iter()
                            .zip(search.iter())
                            .all(|(c1, c2)| *c1 == *c2)
                })
                .map(|index| crate::command_return::success_u32(index as u32))
                .unwrap_or(crate::command_return::failure(ErrorCode::Invalid)),
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }

    fn allow_readonly(
        &self,
        buffer_num: u32,
        buffer: RoAllowBuffer,
    ) -> Result<RoAllowBuffer, (RoAllowBuffer, ErrorCode)> {
        match buffer_num {
            allow_ro::SEARCH => Ok(self.search_buffer.replace(buffer)),
            _ => Err((buffer, ErrorCode::Invalid)),
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

mod command {
    pub const EXISTS: u32 = 0;
    pub const DISCOVER: u32 = 1;
}

mod allow_ro {
    pub const SEARCH: u32 = 0;
}
