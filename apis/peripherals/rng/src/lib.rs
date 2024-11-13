#![no_std]

use core::cell::Cell;
use libtock_platform::{
    share, subscribe::OneId, AllowRw, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

pub struct Rng<S: Syscalls>(S);

impl<S: Syscalls> Rng<S> {
    /// Check if the RNG kernel driver exists
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    /// Request `n` bytes of randomness in an asynchronous way.
    /// Users must first share a buffer slice with the kernel and register an Rng listener
    pub fn get_bytes_async(n: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, GET_BYTES, n, 0).to_result()
    }

    /// Share a buffer slice with the kernel.
    /// Must be used in conjunction with the `share::scope` function
    pub fn allow_buffer<'share>(
        buf: &'share mut [u8],
        allow_rw: share::Handle<AllowRw<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::allow_rw::<DefaultConfig, DRIVER_NUM, 0>(allow_rw, buf)
    }

    pub fn unallow_buffer() {
        S::unallow_rw(DRIVER_NUM, 0)
    }

    /// Register an Rng listener to be called when an upcall is serviced
    /// Must be used in conjunction with the `share::scope` function
    pub fn register_listener<'share, F: Fn(u32)>(
        listener: &'share RngListener<F>,
        subscribe: share::Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    /// Ask to fill the provided `buf` with `n` random bytes.
    /// If `n > buf.len()`, it will simply fill the whole buffer.
    pub fn get_bytes_sync(buf: &mut [u8], n: u32) -> Result<(), ErrorCode> {
        let called = Cell::new(false);
        share::scope::<(AllowRw<S, DRIVER_NUM, 0>, Subscribe<S, DRIVER_NUM, 0>), _, _>(|handle| {
            let (allow_rw, subscribe) = handle.split();

            // Share the provided buffer with the kernel
            S::allow_rw::<DefaultConfig, DRIVER_NUM, 0>(allow_rw, buf)?;

            // Subscribe for an upcall with the kernel
            S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &called)?;

            // Send the command to the kernel driver to fill the allowed_readwrite buffer
            S::command(DRIVER_NUM, GET_BYTES, n, 0).to_result::<(), ErrorCode>()?;

            // Wait for a callback to happen
            while !called.get() {
                S::yield_wait();
            }

            Ok(())
        })
    }
}

/// The provided listener to be called.
/// Interior function operates on the number of random bytes filled into the buffer
pub struct RngListener<F: Fn(u32)>(pub F);

impl<F: Fn(u32)> Upcall<OneId<DRIVER_NUM, 0>> for RngListener<F> {
    fn upcall(&self, _: u32, arg1: u32, _: u32) {
        (self.0)(arg1)
    }
}

// -------------
// DRIVER NUMBER
// -------------
const DRIVER_NUM: u32 = 0x40001;

// ---------------
// COMMAND NUMBERS
// ---------------
const EXISTS: u32 = 0;
const GET_BYTES: u32 = 1;
