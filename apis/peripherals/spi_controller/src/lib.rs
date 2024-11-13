#![no_std]

use core::cell::Cell;
use libtock_platform as platform;
use libtock_platform::allow_rw::AllowRw;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::AllowRo;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

pub struct SpiController<S: Syscalls, C: Config = DefaultConfig>(S, C);

impl<S: Syscalls, C: Config> SpiController<S, C> {
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, spi_controller_cmd::EXISTS, 0, 0).to_result()
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
    pub fn spi_controller_write_read_sync(
        w_buf: &[u8],
        r_buf: &mut [u8],
        len: u32,
    ) -> Result<(), ErrorCode> {
        if len as usize > w_buf.len() || len as usize > r_buf.len() {
            return Err(ErrorCode::NoMem);
        }

        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { rw_allow::READ }>,
                AllowRo<_, DRIVER_NUM, { ro_allow::WRITE }>,
                Subscribe<_, DRIVER_NUM, { subscribe::COMPLETE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, allow_ro, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { rw_allow::READ }>(allow_rw, r_buf)?;
            S::allow_ro::<C, DRIVER_NUM, { ro_allow::WRITE }>(allow_ro, w_buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::COMPLETE }>(subscribe, &called)?;

            S::command(DRIVER_NUM, spi_controller_cmd::READ_WRITE_BYTES, len, 0)
                .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, status, _)) = called.get() {
                    assert_eq!(r0, len);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }

    pub fn spi_controller_write_sync(w_buf: &[u8], len: u32) -> Result<(), ErrorCode> {
        if len as usize > w_buf.len() {
            return Err(ErrorCode::NoMem);
        }

        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { ro_allow::WRITE }>,
                Subscribe<_, DRIVER_NUM, { subscribe::COMPLETE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();
            S::allow_ro::<C, DRIVER_NUM, { ro_allow::WRITE }>(allow_ro, w_buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::COMPLETE }>(subscribe, &called)?;

            S::command(DRIVER_NUM, spi_controller_cmd::READ_WRITE_BYTES, len, 0)
                .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, status, _)) = called.get() {
                    assert_eq!(r0, len);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }

    pub fn spi_controller_read_sync(r_buf: &mut [u8], len: u32) -> Result<(), ErrorCode> {
        if len as usize > r_buf.len() {
            return Err(ErrorCode::NoMem);
        }

        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { rw_allow::READ }>,
                Subscribe<_, DRIVER_NUM, { subscribe::COMPLETE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { rw_allow::READ }>(allow_rw, r_buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::COMPLETE }>(subscribe, &called)?;

            S::command(DRIVER_NUM, spi_controller_cmd::READ_BYTES, len, 0)
                .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, status, _)) = called.get() {
                    assert_eq!(r0, len);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }

    pub fn spi_controller_inplace_write_read_sync(
        r_buf: &mut [u8],
        len: u32,
    ) -> Result<(), ErrorCode> {
        if len as usize > r_buf.len() {
            return Err(ErrorCode::NoMem);
        }

        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { rw_allow::READ }>,
                Subscribe<_, DRIVER_NUM, { subscribe::COMPLETE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { rw_allow::READ }>(allow_rw, r_buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::COMPLETE }>(subscribe, &called)?;

            S::command(
                DRIVER_NUM,
                spi_controller_cmd::INPLACE_READ_WRITE_BYTES,
                len,
                0,
            )
            .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((r0, status, _)) = called.get() {
                    assert_eq!(r0, len);
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }
}

/// System call configuration trait for `SpiController`.
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
const DRIVER_NUM: u32 = 0x20001;

#[allow(unused)]
mod subscribe {
    pub const COMPLETE: u32 = 0;
}

#[allow(unused)]
mod ro_allow {
    pub const WRITE: u32 = 0;
}

#[allow(unused)]
mod rw_allow {
    pub const READ: u32 = 0;
}

#[allow(unused)]
mod spi_controller_cmd {
    pub const EXISTS: u32 = 0;
    pub const READ_WRITE_BYTES: u32 = 2;
    pub const SET_BAUD: u32 = 5;
    pub const GET_BAUD: u32 = 6;
    pub const SET_PHASE: u32 = 7;
    pub const GET_PHASE: u32 = 8;
    pub const SET_POLARITY: u32 = 9;
    pub const GET_POLARITY: u32 = 10;
    pub const READ_BYTES: u32 = 11;
    pub const INPLACE_READ_WRITE_BYTES: u32 = 12;
}
