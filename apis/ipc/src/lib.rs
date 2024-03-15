#![no_std]

use crate::share::Handle;
use core::cell::Cell;
use libtock_platform as platform;
use libtock_platform::share;
use libtock_platform::AllowRo;
use libtock_platform::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

pub struct IPC<S: Syscalls, C: Config = DefaultConfig>(S, C);

impl<S: Syscalls, C: Config> IPC<S, C> {
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, i2c_master_cmd::EXISTS, 0, 0).to_result()
    }

    /// # Summary
    ///
    /// Performs service discovery
    ///
    /// Retrieves the process identifier of the process with the given package
    /// name, or a negative value on error.
    ///
    /// # Parameter
    ///
    /// * `pkg_name`: The package name of this service
    ///
    /// # Returns
    /// On success: Ok(svc_id)
    ///             Where `svc_id` is the process id of the service
    /// On failure: Err(ErrorCode)
    pub fn discover(pkg_name: &[u8]) -> Result<usize, ErrorCode> {
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { ro_allow::BUFFER }>,
                Subscribe<_, DRIVER_NUM, 0>,
            ),
            _,
            _,
        >(
            |handle: Handle<
                '_,
                (
                    AllowRo<'_, S, DRIVER_NUM, { ro_allow::BUFFER }>,
                    Subscribe<'_, _, DRIVER_NUM, 0>,
                ),
            >| {
                let (allow_ro, _subscribe): (
                    Handle<'_, AllowRo<'_, S, DRIVER_NUM, { ro_allow::BUFFER }>>,
                    Handle<'_, Subscribe<'_, S, DRIVER_NUM, 0>>,
                ) = handle.split();
                S::allow_ro::<C, DRIVER_NUM, { ro_allow::BUFFER }>(allow_ro, pkg_name)?;

                let svc_id: u32 = S::command(DRIVER_NUM, i2c_master_cmd::DISCOVER, 0, 0)
                    .to_result::<u32, ErrorCode>()?;

                Ok(svc_id as usize)
            },
        )
    }

    pub fn wait_for_client_notify(pkg_name: &[u8]) -> Result<usize, ErrorCode> {
        let svc_id = IPC::<S, C>::discover(pkg_name)?;

        let called: Cell<Option<(u32, u32, u32)>> = Cell::new(None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { ro_allow::BUFFER }>,
                Subscribe<_, DRIVER_NUM, 0>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();
            S::allow_ro::<C, DRIVER_NUM, { ro_allow::BUFFER }>(allow_ro, pkg_name)?;
            S::ipc_subscribe::<_, _, C, DRIVER_NUM>(subscribe, svc_id as u32, &called)?;

            let svc_id: u32 = S::command(DRIVER_NUM, i2c_master_cmd::DISCOVER, 0, 0).to_result()?;

            loop {
                S::yield_wait();
                if let Some((pid, len, _)) = called.get() {
                    assert_eq!(pid, svc_id);
                    return Ok(len as usize);
                }
            }
        })
    }
}

/// System call configuration trait for `IPC`.
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
const DRIVER_NUM: u32 = 0x10000;

/// Ids for read-only allow buffers
#[allow(unused)]
mod ro_allow {
    pub const BUFFER: u32 = 0;
}

#[allow(unused)]
mod i2c_master_cmd {
    pub const EXISTS: u32 = 0;
    pub const DISCOVER: u32 = 1;
    pub const SERVICE_NOTIFY: u32 = 2;
    pub const CLIENT_NOTIFY: u32 = 3;
}
