#![no_std]

#[cfg(test)]
mod tests;

use core::cell::Cell;
use libtock_platform::{share, AllowRw, DefaultConfig, ErrorCode, Subscribe, Syscalls};

const DRIVER_NUM: u32 = 0x40001;
const EXISTS: u32 = 0;
const GET_BYTES: u32 = 1;

pub struct Rng<S: Syscalls>(S);

impl<S: Syscalls> Rng<S> {
    /// Check if the RNG kernel driver exists
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    /// Ask to fill the provided `buf` with `n` random bytes.
    /// If `n > buf.len()`, it will simply fill the whole buffer.
    pub fn get_bytes_sync(buf: &mut [u8], n: u32) -> Result<(), ErrorCode> {
        let called = Cell::new(false);
        share::scope::<(AllowRw<S, DRIVER_NUM, 0>, Subscribe<S, DRIVER_NUM, 0>), _, _>(|handle| {
            let (allow_ro, subscribe) = handle.split();

            // Share the provided buffer with the kernel
            S::allow_rw::<DefaultConfig, DRIVER_NUM, 0>(allow_ro, buf)?;

            // Subscribe for an upcall with the kernel
            S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &called)?;

            // Send the command to the kernel driver to fill the allowed_readwrite buffer
            S::command(DRIVER_NUM, GET_BYTES, n, 0).to_result()?;

            // Wait for a callback to happen
            while !called.get() {
                S::yield_wait();
            }

            Ok(())
        })
    }
}
