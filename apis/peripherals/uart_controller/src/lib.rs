#![no_std]

use core::cell::Cell;
use libtock_platform as platform;
use libtock_platform::allow_rw::AllowRw;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::AllowRo;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

pub struct UartController<S: Syscalls, C: Config = DefaultConfig>(S, C);

impl<S: Syscalls, C: Config> UartController<S, C> {
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, cmd::EXISTS, 0, 0).to_result()
    }

    pub fn is_supported_port(port: u32) -> bool {
        matches!(port, 0 | 1)
    }

    pub fn uart_controller_write_sync(port: u32, w_buf: &[u8], len: u32) -> Result<(), ErrorCode> {
        if !Self::is_supported_port(port) {
            return Err(ErrorCode::Invalid);
        }
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
            S::command(DRIVER_NUM, cmd::WRITE, len, port).to_result::<(), ErrorCode>()?;
            loop {
                S::yield_wait();
                if let Some((r0, status, _)) = called.get() {
                    assert_eq!(r0, len);
                    return match status {
                        0 => Ok(()),
                        e => Err(e.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }

    pub fn uart_controller_read_sync(
        port: u32,
        r_buf: &mut [u8],
        len: u32,
    ) -> Result<(), ErrorCode> {
        if !Self::is_supported_port(port) {
            return Err(ErrorCode::Invalid);
        }
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
            S::command(DRIVER_NUM, cmd::READ, len, port).to_result::<(), ErrorCode>()?;
            loop {
                S::yield_wait();
                if let Some((r0, status, _)) = called.get() {
                    assert_eq!(r0, len);
                    return match status {
                        0 => Ok(()),
                        e => Err(e.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        })
    }
}

pub trait Config:
    platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config
{
}
impl<T: platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config>
    Config for T
{
}

const DRIVER_NUM: u32 = 0x22;
mod subscribe {
    pub const COMPLETE: u32 = 0;
}
mod ro_allow {
    pub const WRITE: u32 = 1;
}
mod rw_allow {
    pub const READ: u32 = 1;
}
mod cmd {
    pub const EXISTS: u32 = 0;
    pub const WRITE: u32 = 1;
    pub const READ: u32 = 2;
}

#[cfg(test)]
mod tests;
