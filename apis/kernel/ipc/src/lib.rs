#![no_std]

use libtock_platform as platform;
use libtock_platform::share;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

pub struct Ipc<S: Syscalls, C: Config = DefaultConfig>(S, C);

impl<S: Syscalls, C: Config> Ipc<S, C> {
    /// Check if the IPC kernel driver exists
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::EXISTS, 0, 0).to_result()
    }

    /// Request the service ID of an IPC service by package name
    pub fn discover(pkg_name: &[u8]) -> Result<u32, ErrorCode> {
        share::scope(|allow_search| {
            // Share the package name buffer with the kernel to search for
            S::allow_ro::<C, DRIVER_NUM, { allow_ro::SEARCH }>(allow_search, pkg_name)?;

            // Send the command to the kernel driver to retrieve the service id for the
            // corresponding IPC service, if it exists
            S::command(DRIVER_NUM, command::DISCOVER, 0, 0).to_result()
        })
    }
}

/// System call configuration trait for `Ipc`.
pub trait Config: platform::allow_ro::Config {}
impl<T: platform::allow_ro::Config> Config for T {}

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
