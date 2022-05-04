#![no_std]

use core::fmt;
use core::marker::PhantomData;
use libtock_platform as platform;
use libtock_platform::allow_ro::AllowRo;
use libtock_platform::allow_rw::AllowRw;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

const CONSOLE_DRIVER_NUM: u32 = 1;

/// The console driver.
///
/// It allows libraries to pass strings to the kernel's console driver.
///
/// # Example
/// ```ignore
/// use libtock2::Console;
///
/// // Writes "foo", followed by a newline, to the console
/// let mut writer = Console::writer();
/// writeln!(writer, foo).unwrap();
/// ```
pub type Console<S, C = DefaultConfig> = Serial<S, C, CONSOLE_DRIVER_NUM>;

pub struct Serial<
    S: Syscalls,
    C: platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config = DefaultConfig,
    const DRIVER_NUM: u32 = CONSOLE_DRIVER_NUM,
>(S, C);

impl<S: Syscalls, C: platform::allow_ro::Config + platform::allow_rw::Config  + platform::subscribe::Config, const DRIVER_NUM: u32> Serial<S, C, DRIVER_NUM> {
    /// Run a check against the console capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn driver_check() -> bool {
        S::command(DRIVER_NUM, command::DRIVER_CHECK, 0, 0).is_success()
    }

    /// Writes bytes, returns count of bytes written.
    pub fn write(s: &[u8]) -> Result<u32, ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32,)>::None);
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

            S::command(DRIVER_NUM, command::WRITE, s.len() as u32, 0).to_result()?;

            loop {
                S::yield_wait();
                if let Some((bytes_read_count,)) = called.get() {
                    return Ok(bytes_read_count);
                }
            }
        })
    }

    /// Writes all bytes of a slice.
    /// This is an alternative to `fmt::Write::write`
    /// because this can actually return an error code.
    /// It's makes only one `subscribe` call,
    /// as opposed to calling `write` in a loop.
    pub fn write_all(s: &[u8]) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32,)>::None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { allow_ro::WRITE }>,
                Subscribe<_, DRIVER_NUM, { subscribe::WRITE }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::WRITE }>(subscribe, &called)?;

            let mut remaining = s.len();
            while remaining > 0 {
                S::allow_ro::<C, DRIVER_NUM, { allow_ro::WRITE }>(
                    allow_ro,
                    &s[(s.len() - remaining)..],
                )?;

                S::command(DRIVER_NUM, command::WRITE, remaining as u32, 0).to_result()?;

                loop {
                    S::yield_wait();
                    if let Some((bytes_read_count,)) = called.get() {
                        remaining -= bytes_read_count as usize;
                        called.set(None);
                        break;
                    }
                }
            }
            Ok(())
        })
    }
    
    /// Reads bytes
    /// Reads from the device and writes to `buf`, starting from index 0.
    /// No special guarantees about when the read stops.
    /// Returns count of bytes written to `buf`.
    pub fn read(buf: &mut [u8]) -> (u32, Result<(), ErrorCode>) {
        let called = core::cell::Cell::new(Option::<(u32,u32)>::None);
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
            S::command(DRIVER_NUM, command::READ, len as u32, 0).to_result()?;

            loop {
                S::yield_wait();
                if let Some((status, bytes_pushed_count)) = called.get() {
                    bytes_received = bytes_pushed_count;
                    return match status {
                        0 => Ok(()),
                        other => Err(status.try_into().unwrap_or(ErrorCode::Fail)),
                    }
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
        Console::<S>::write_all(s.as_bytes()).map_err(|_e| fmt::Error)
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

// Command IDs
#[allow(unused)]
mod command {
    pub const DRIVER_CHECK: u32 = 0;
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