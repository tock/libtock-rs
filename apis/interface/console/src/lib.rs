#![no_std]

use core::cell::Cell;
use core::fmt;
use core::marker::PhantomData;
use libtock_platform as platform;
use libtock_platform::allow_ro::AllowRo;
use libtock_platform::allow_rw::AllowRw;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

/// The console driver.
///
/// It allows libraries to pass strings to the kernel's console driver.
///
/// # Example
/// ```ignore
/// use libtock::Console;
///
/// // Writes "foo", followed by a newline, to the console
/// let mut writer = Console::writer();
/// writeln!(writer, foo).unwrap();
/// ```
pub struct Console<S: Syscalls, C: Config = DefaultConfig>(S, C);

impl<S: Syscalls, C: Config> Console<S, C> {
    /// Run a check against the console capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn exists() -> bool {
        S::command(DRIVER_NUM, command::EXISTS, 0, 0).is_success()
    }

    /// Writes bytes.
    /// This is an alternative to `fmt::Write::write`
    /// because this can actually return an error code.
    pub fn write(s: &[u8]) -> Result<(), ErrorCode> {
        let called: Cell<Option<(u32,)>> = Cell::new(None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { allow_ro::WRITE }>,
                Subscribe<_, DRIVER_NUM, { subscribe::WRITE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();

            S::allow_ro::<C, DRIVER_NUM, { allow_ro::WRITE }>(allow_ro, s)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::WRITE }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::WRITE, s.len() as u32, 0)
                .to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((_,)) = called.get() {
                    return Ok(());
                }
            }
        })
    }

    /// Reads bytes
    /// Reads from the device and writes to `buf`, starting from index 0.
    /// No special guarantees about when the read stops.
    /// Returns count of bytes written to `buf`.
    pub fn read(buf: &mut [u8]) -> (usize, Result<(), ErrorCode>) {
        let called: Cell<Option<(u32, u32)>> = Cell::new(None);
        let mut bytes_received = 0;
        let r = share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { allow_rw::READ }>,
                Subscribe<_, DRIVER_NUM, { subscribe::READ }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();
            let len = buf.len();
            S::allow_rw::<C, DRIVER_NUM, { allow_rw::READ }>(allow_rw, buf)?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::READ }>(subscribe, &called)?;

            // When this fails, `called` is guaranteed unmodified,
            // because upcalls are never processed until we call `yield`.
            S::command(DRIVER_NUM, command::READ, len as u32, 0).to_result::<(), ErrorCode>()?;

            loop {
                S::yield_wait();
                if let Some((status, bytes_pushed_count)) = called.get() {
                    bytes_received = bytes_pushed_count as usize;
                    return match status {
                        0 => Ok(()),
                        e_status => Err(e_status.try_into().unwrap_or(ErrorCode::Fail)),
                    };
                }
            }
        });
        (bytes_received, r)
    }

    pub fn writer() -> ConsoleWriter<S> {
        ConsoleWriter {
            syscalls: Default::default(),
        }
    }
}

pub struct ConsoleWriter<S: Syscalls> {
    syscalls: PhantomData<S>,
}

impl<S: Syscalls> fmt::Write for ConsoleWriter<S> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        Console::<S>::write(s.as_bytes()).map_err(|_e| fmt::Error)
    }
}

/// System call configuration trait for `Console`.
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

const DRIVER_NUM: u32 = 0x1;

// Command IDs
#[allow(unused)]
mod command {
    pub const EXISTS: u32 = 0;
    pub const WRITE: u32 = 1;
    pub const READ: u32 = 2;
    pub const ABORT: u32 = 3;
}

#[allow(unused)]
mod subscribe {
    pub const WRITE: u32 = 1;
    pub const READ: u32 = 2;
}

mod allow_ro {
    pub const WRITE: u32 = 1;
}

mod allow_rw {
    pub const READ: u32 = 1;
}
