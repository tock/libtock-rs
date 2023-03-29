#![no_std]

use libtock_platform::{ErrorCode, Syscalls};

pub struct Proximity<S: Syscalls>(S);

impl<S: Syscalls> Proximity<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }
}

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60005;

// Command IDs

const EXISTS: u32 = 0;
const READ: u32 = 1;
const READ_ON_INT: u32 = 2;
