#![no_std]

use core::cell::Cell;
use libtock_platform as platform;
use libtock_platform::allow_ro::AllowRo;
use libtock_platform::allow_rw::AllowRw;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

/// The key-value driver.
///
/// It provides access to a key-value store.
pub struct KeyValue<S: Syscalls, C: Config = DefaultConfig>(S, C);

impl<S: Syscalls, C: Config> KeyValue<S, C> {
    /// Run a check against the key-value capsule to ensure it is present.
    #[inline(always)]
    pub fn exists() -> bool {
        S::command(DRIVER_NUM, command::DRIVER_CHECK, 0, 0).is_success()
    }

    /// Get a key-value object from the `key`.
    pub fn get(key: &[u8], value: &mut [u8]) -> Result<u32, ErrorCode> {
        let called: Cell<Option<Result<(u32,), ErrorCode>>> = Cell::new(None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { allow_ro::KEY }>,
                AllowRw<_, DRIVER_NUM, { allow_rw::VALUE_READ }>,
                Subscribe<_, DRIVER_NUM, { subscribe::CALLBACK }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_key, allow_value, subscribe) = handle.split();

            S::allow_ro::<C, DRIVER_NUM, { allow_ro::KEY }>(allow_key, key)?;
            S::allow_rw::<C, DRIVER_NUM, { allow_rw::VALUE_READ }>(allow_value, value)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::CALLBACK }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::GET, 0, 0).to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some(ret) = called.get() {
                    return ret.map(|(arg0,)| arg0);
                }
            }
        })
    }

    /// Set a key-value object for the `key`.
    fn insert(command_num: u32, key: &[u8], value: &[u8]) -> Result<(), ErrorCode> {
        let called: Cell<Option<Result<(), ErrorCode>>> = Cell::new(None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { allow_ro::KEY }>,
                AllowRo<_, DRIVER_NUM, { allow_ro::VALUE_WRITE }>,
                Subscribe<_, DRIVER_NUM, { subscribe::CALLBACK }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_key, allow_value, subscribe) = handle.split();

            S::allow_ro::<C, DRIVER_NUM, { allow_ro::KEY }>(allow_key, key)?;
            S::allow_ro::<C, DRIVER_NUM, { allow_ro::VALUE_WRITE }>(allow_value, value)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::CALLBACK }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command_num, 0, 0).to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some(ret) = called.get() {
                    return ret;
                }
            }
        })
    }

    /// Set a key-value object for the `key`.
    pub fn set(key: &[u8], value: &[u8]) -> Result<(), ErrorCode> {
        Self::insert(command::SET, key, value)
    }

    /// Set a key-value object for the `key`.
    pub fn add(key: &[u8], value: &[u8]) -> Result<(), ErrorCode> {
        Self::insert(command::ADD, key, value)
    }

    /// Set a key-value object for the `key`.
    pub fn update(key: &[u8], value: &[u8]) -> Result<(), ErrorCode> {
        Self::insert(command::UPDATE, key, value)
    }

    /// Delete a key-value object by `key`.
    pub fn delete(key: &[u8]) -> Result<(), ErrorCode> {
        let called: Cell<Option<Result<(), ErrorCode>>> = Cell::new(None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { allow_ro::KEY }>,
                Subscribe<_, DRIVER_NUM, { subscribe::CALLBACK }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_key, subscribe) = handle.split();

            S::allow_ro::<C, DRIVER_NUM, { allow_ro::KEY }>(allow_key, key)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::CALLBACK }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::DELETE, 0, 0).to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some(ret) = called.get() {
                    return ret;
                }
            }
        })
    }
}

/// System call configuration trait for `KeyValue`.
pub trait Config:
    platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config
{
}
impl<T: platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config>
    Config for T
{
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x50003;

// Command IDs
#[allow(unused)]
mod command {
    pub const DRIVER_CHECK: u32 = 0;
    pub const GET: u32 = 1;
    pub const SET: u32 = 2;
    pub const DELETE: u32 = 3;
    pub const ADD: u32 = 4;
    pub const UPDATE: u32 = 5;
}

#[allow(unused)]
mod subscribe {
    pub const CALLBACK: u32 = 0;
}

mod allow_ro {
    pub const KEY: u32 = 0;
    pub const VALUE_WRITE: u32 = 1;
}

mod allow_rw {
    pub const VALUE_READ: u32 = 0;
}
