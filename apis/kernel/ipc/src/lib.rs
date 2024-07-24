#![no_std]

use libtock_platform::Syscalls;

/// TODO: add IPC API documentation here

pub struct Ipc<S: Syscalls>(S);

impl<S: Syscalls> Ipc<S> {
    /// Run a check against the IPC capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn exists() -> bool {
        S::command(DRIVER_NUM, EXISTS, 0, 0).is_success()
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
