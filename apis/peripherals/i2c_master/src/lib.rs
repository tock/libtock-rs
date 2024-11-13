#![no_std]

use core::cell::Cell;
use libtock_platform as platform;
use libtock_platform::allow_rw::AllowRw;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

pub struct I2CMaster<S: Syscalls, C: Config = DefaultConfig>(S, C);

impl<S: Syscalls, C: Config> I2CMaster<S, C> {
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, i2c_master_cmd::EXISTS, 0, 0).to_result()
    }

    /// # Summary
    ///
    /// Perform an I2C write followed by a read.
    ///
    /// TODO: Add async support
    ///
    /// # Parameter
    ///
    /// * `addr`: Slave device address
    /// * `buf`: Buffer
    /// * `w_len`: Number of bytes to write from @w_buf
    /// * `r_len`: Number of bytes to read into @r_buf
    ///
    /// # Returns
    /// On success: Returns Ok(())
    /// On failure: Err(ErrorCode)
    pub fn i2c_master_write_read_sync(
        addr: u16,
        buf: &mut [u8],
        w_len: u16,
        r_len: u16,
    ) -> Result<(), ErrorCode> {
        if w_len as usize > buf.len() || r_len as usize > buf.len() {
            return Err(ErrorCode::NoMem);
        }
        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        let cmd_arg0: u32 = (w_len as u32) << 8 | addr as u32;
        share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { rw_allow::MASTER }>,
                Subscribe<_, DRIVER_NUM, { subscribe::MASTER_WRITE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { rw_allow::MASTER }>(allow_rw, buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::MASTER_READ_WRITE }>(
                subscribe, &called,
            )?;

            S::command(
                DRIVER_NUM,
                i2c_master_cmd::MASTER_WRITE,
                cmd_arg0,
                r_len.into(),
            )
            .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, status, _)) = called.get() {
                    assert_eq!(r0, 0);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }

    /// # Summary
    ///
    /// Write to an I2C device the data from the buffer pointed by @buf. This function is
    /// synchronous and returns only when the operation has completed.
    ///
    /// TODO: Add async support
    ///
    /// # Parameter
    ///
    /// * `addr`: Slave device address
    /// * `buf`: Storage buffer, this should be bigger than @len
    /// * `len`: Number of bytes to read into @buf
    ///
    /// # Returns
    /// On success: Returns Ok(())
    /// On failure: Err(ErrorCode)
    pub fn i2c_master_write_sync(addr: u16, buf: &mut [u8], len: u16) -> Result<(), ErrorCode> {
        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { rw_allow::MASTER }>,
                Subscribe<_, DRIVER_NUM, { subscribe::MASTER_WRITE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { rw_allow::MASTER }>(allow_rw, buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::MASTER_WRITE }>(subscribe, &called)?;

            S::command(
                DRIVER_NUM,
                i2c_master_cmd::MASTER_WRITE,
                addr.into(),
                len.into(),
            )
            .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, status, _)) = called.get() {
                    assert_eq!(r0, 0);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }

    /// # Summary
    ///
    /// Read from an I2C device the data to the buffer pointed by @buf. This function is
    /// synchronous and returns only when the operation has completed.
    ///
    /// TODO: Add async support
    ///
    /// # Parameter
    ///
    /// * `addr`: Slave device address
    /// * `buf`: Storage buffer, this should be bigger than @len
    /// * `len`: Number of bytes to read into @buf
    ///
    /// # Returns
    /// On success: Returns Ok(())
    /// On failure: Err(ErrorCode)
    pub fn i2c_master_read_sync(addr: u16, buf: &mut [u8], len: u16) -> Result<(), ErrorCode> {
        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { rw_allow::MASTER }>,
                Subscribe<_, DRIVER_NUM, { subscribe::MASTER_READ }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { rw_allow::MASTER }>(allow_rw, buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::MASTER_READ }>(subscribe, &called)?;

            S::command(
                DRIVER_NUM,
                i2c_master_cmd::MASTER_READ,
                addr.into(),
                len.into(),
            )
            .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, status, _)) = called.get() {
                    assert_eq!(r0, 0);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }
}

/// System call configuration trait for `I2CMaster`.
pub trait Config:
    platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config
{
}
impl<T: platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config>
    Config for T
{
}

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------
const DRIVER_NUM: u32 = 0x20003;

#[allow(unused)]
mod subscribe {
    pub const MASTER_READ: u32 = 0;
    pub const MASTER_WRITE: u32 = 0;
    pub const MASTER_READ_WRITE: u32 = 0;
}

/// Ids for read-write allow buffers
#[allow(unused)]
mod rw_allow {
    pub const MASTER: u32 = 1;
}

#[allow(unused)]
mod i2c_master_cmd {
    pub const EXISTS: u32 = 0;
    pub const MASTER_WRITE: u32 = 1;
    pub const MASTER_READ: u32 = 2;
    pub const MASTER_WRITE_READ: u32 = 3;
}
