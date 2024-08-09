//! Fake implementation of the IPC API.
//!
//! Like the real IPC API, `Ipc` coordinates interprocess communication
//! between a set of fake processes. It provides a function `as_process`
//! used to mock interfacing with the IPC API as different processes.
//!
//! Process indexes (what the present IPC kernel driver uses to identify IPC
//! services and clients) are assigned implicitly based on the order of
//! processes passed in to `Ipc::new`.

use libtock_platform::{CommandReturn, ErrorCode};
use std::cell::{Cell, RefCell};

use crate::{DriverInfo, DriverShareRef, RoAllowBuffer, RwAllowBuffer};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Process {
    pkg_name: Vec<u8>,
    process_id: u32,
}

impl Process {
    pub fn new(pkg_name: &[u8], process_id: u32) -> Self {
        Process {
            pkg_name: Vec::from(pkg_name),
            process_id,
        }
    }
}

enum IpcProcessType {
    Service,
    Client,
}

pub struct Ipc<const NUM_PROCS: usize> {
    processes: [Process; NUM_PROCS],
    current_index: Cell<Option<u32>>,
    search_buffer: RefCell<RoAllowBuffer>,
    share_buffers: [RefCell<RwAllowBuffer>; NUM_PROCS],
    share_ref: DriverShareRef,
}

impl<const NUM_PROCS: usize> Ipc<NUM_PROCS> {
    pub fn new(processes: &[Process; NUM_PROCS]) -> std::rc::Rc<Ipc<NUM_PROCS>> {
        std::rc::Rc::new(Ipc {
            processes: Vec::from(processes).try_into().unwrap(),
            current_index: Default::default(),
            search_buffer: Default::default(),
            share_buffers: std::array::from_fn(|_| Default::default()),
            share_ref: Default::default(),
        })
    }
    pub fn as_process<F: Fn()>(&self, process_id: u32, process_fn: F) -> Result<(), ErrorCode> {
        let index = self
            .processes
            .iter()
            .position(|process| process.process_id == process_id)
            .ok_or(ErrorCode::Invalid)?;
        self.current_index.replace(Some(index as u32));
        process_fn();
        self.current_index.set(None);
        Ok(())
    }
}

impl<const NUM_PROCS: usize> crate::fake::SyscallDriver for Ipc<NUM_PROCS> {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(NUM_PROCS as u32)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn command(&self, command_num: u32, to_index: u32, _argument1: u32) -> CommandReturn {
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
            command::SERVICE_NOTIFY => self.notify(IpcProcessType::Service, to_index),
            command::CLIENT_NOTIFY => self.notify(IpcProcessType::Client, to_index),
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

    fn allow_readwrite(
        &self,
        buffer_num: u32,
        buffer: RwAllowBuffer,
    ) -> Result<RwAllowBuffer, (RwAllowBuffer, ErrorCode)> {
        if let Some(search_buffer) = self.share_buffers.get(buffer_num as usize) {
            Ok(search_buffer.replace(buffer))
        } else {
            Err((buffer, ErrorCode::Invalid))
        }
    }
}

impl<const NUM_PROCS: usize> Ipc<NUM_PROCS> {
    fn notify(&self, target: IpcProcessType, to_index: u32) -> CommandReturn {
        if to_index >= NUM_PROCS as u32 {
            return crate::command_return::failure(ErrorCode::Invalid);
        }

        let from_index = self.current_index.get().expect("No current application");
        let service_index = match target {
            IpcProcessType::Service => to_index,
            IpcProcessType::Client => from_index,
        };

        let share_buffer = self
            .share_buffers
            .get(service_index as usize)
            .expect("Unable to access share buffer");
        let share_len = share_buffer.borrow().len() as u32;
        let share_ptr = match share_len {
            0 => 0,
            _ => share_buffer.borrow().as_ptr() as u32,
        };
        self.share_ref
            .schedule_upcall(service_index, (from_index, share_len, share_ptr))
            .expect("Unable to schedule upcall {}");

        crate::command_return::success()
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
    pub const SERVICE_NOTIFY: u32 = 2;
    pub const CLIENT_NOTIFY: u32 = 3;
}

// Read-only allow numbers
mod allow_ro {
    pub const SEARCH: u32 = 0;
}
