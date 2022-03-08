#![no_std]

use core::fmt;
use core::marker::PhantomData;
use libtock_platform as platform;
use libtock_platform::allow_ro::AllowRo;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

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
pub struct Console<
    S: Syscalls,
    C: platform::allow_ro::Config + platform::subscribe::Config = DefaultConfig,
>(S, C);

impl<S: Syscalls, C: platform::allow_ro::Config + platform::subscribe::Config> Console<S, C> {
    /// Run a check against the console capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn driver_check() -> bool {
        S::command(DRIVER_NUM, command::DRIVER_CHECK, 0, 0).is_success()
    }

    /// Writes bytes.
    /// This is an alternative to `fmt::Write::write`
    /// because this can actually return an error code.
    pub fn write(s: &[u8]) -> Result<(), ErrorCode> {
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
                if let Some((_,)) = called.get() {
                    return Ok(());
                }
            }
        })
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

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 1;

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
    use libtock_platform::subscribe;
    pub const WRITE: u32 = 1;
    pub const READ: u32 = 2;
}

mod allow_ro {
    pub const WRITE: u32 = 1;
}
