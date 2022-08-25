#![no_std]
/*! Synchronous block storage driver.

Useful mainly for testing the syscall driver.
*/
// Written by dcz <gihuac.dcz@porcupinefactory.org>

use libtock_platform as platform;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

#[derive(Debug)]
pub struct Geometry {
    pub write_block_size: u32,
    pub erase_block_size: u32,
}

impl Geometry {
    pub fn get_erase_block_containing(&self, address: u64) -> u32 {
        (address / self.erase_block_size as u64) as u32
    }

    pub fn get_write_block_containing(&self, address: u64) -> u32 {
        (address / self.write_block_size as u64) as u32
    }

    pub fn get_address_of_write_block(&self, idx: u32) -> u64 {
        idx as u64 * self.write_block_size as u64
    }
}

/// The block storage driver.
///
/// It allows libraries to access a block storage device.
///
/// # Example
/// ```ignore
/// use libtock2::BlockStorage;
///
/// let Geometry::{write_block_size, erase_block_size}
///     = BlockStorage::get_geometry();
/// let mut buf = vec![0, write_block_size];
/// // Reads block number 43 into `buf`
/// BlockStorage::read(43, &mut buf).unwrap();
/// ```
pub struct BlockStorage<
    S: Syscalls,
    C: platform::allow_ro::Config
        + platform::allow_rw::Config
        + platform::subscribe::Config
        = DefaultConfig
>(S, C);

impl<
        S: Syscalls,
        C: platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config,
    > BlockStorage<S, C>
{
    /// Run a check against the low-level capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn driver_check() -> bool {
        S::command(DRIVER_NUM, command::DRIVER_CHECK, 0, 0).is_success()
    }

    pub fn get_geometry() -> Geometry {
        let (write_block_size, erase_block_size) = S::command(DRIVER_NUM, command::GEOMETRY, 0, 0)
            .get_success_2_u32()
            .unwrap();
        Geometry {
            write_block_size,
            erase_block_size,
        }
    }

    pub fn get_size() -> u64 {
        S::command(DRIVER_NUM, command::SIZE, 0, 0)
            .get_success_u64()
            .unwrap()
    }

    pub fn read(block_idx: u32, buf: &mut [u8]) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32, u32)>::None);
        share::scope::<
            (
                platform::AllowRw<_, DRIVER_NUM, { allow_rw::READ }>,
                Subscribe<_, DRIVER_NUM, { subscribe::READ }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();

            S::allow_rw::<C, DRIVER_NUM, { allow_rw::READ }>(allow_rw, buf)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::READ }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::READ, block_idx, 1).to_result()?;

            loop {
                S::yield_wait();
                if let Some((is_error, errno)) = called.get() {
                    return match is_error {
                        0 => Ok(()),
                        _ => Err(errno.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }

    pub fn write(block_idx: u32, buf: &[u8]) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32, u32)>::None);
        share::scope::<
            (
                platform::AllowRo<_, DRIVER_NUM, { allow_ro::WRITE }>,
                Subscribe<_, DRIVER_NUM, { subscribe::WRITE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();

            S::allow_ro::<C, DRIVER_NUM, { allow_ro::WRITE }>(allow_ro, buf)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::WRITE }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::WRITE, block_idx, 1).to_result()?;

            loop {
                S::yield_wait();
                if let Some((is_error, errno)) = called.get() {
                    return match is_error {
                        0 => Ok(()),
                        _ => Err(errno.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }

    pub fn erase(block_idx: u32) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32, u32)>::None);
        share::scope::<Subscribe<_, DRIVER_NUM, { subscribe::ERASE }>, _, _>(|handle| {
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::ERASE }>(handle, &called)?;

            S::command(DRIVER_NUM, command::ERASE, block_idx, 1).to_result()?;

            loop {
                S::yield_wait();
                if let Some((is_error, errno)) = called.get() {
                    return match is_error {
                        0 => Ok(()),
                        _ => Err(errno.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }
}

//#[cfg(test)]
//mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x50004;

// Command IDs
#[allow(unused)]
mod command {
    pub const DRIVER_CHECK: u32 = 0;
    pub const SIZE: u32 = 1;
    pub const GEOMETRY: u32 = 2;
    pub const READ_RANGE: u32 = 3;
    pub const READ: u32 = 4;
    pub const ERASE: u32 = 5;
    pub const WRITE: u32 = 6;
}

#[allow(unused)]
mod subscribe {
    pub const READ: u32 = 0;
    pub const ERASE: u32 = 1;
    pub const WRITE: u32 = 2;
}

mod allow_ro {
    pub const WRITE: u32 = 0;
}

mod allow_rw {
    pub const READ: u32 = 0;
}
