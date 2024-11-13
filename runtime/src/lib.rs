//! `libtock_runtime` provides the runtime for Tock process binaries written in
//! Rust as well as interfaces to Tock's system calls.
//!
//! `libtock_runtime` is designed for statically-compiled binaries, and needs to
//! know the location (in non-volatile memory and RAM) at which the process will
//! execute. It reads the `LIBTOCK_PLATFORM` variable to determine what location
//! to build for (see the `layouts/` directory to see what platforms are
//! available). It expects the following cargo config options to be set (e.g. in
//! `.cargo/config.toml`):
//! ```
//! [build]
//! rustflags = [
//!     "-C", "relocation-model=static",
//!     "-C", "link-arg=-Tlayout.ld",
//! ]
//! ```
//! If a process binary wants to support another platform, it can set the
//! `no_auto_layout` feature on `libtock_runtime` to disable this functionality
//! and provide its own layout file.

#![no_std]
#![warn(unsafe_op_in_unsafe_fn)]

pub mod startup;

/// TockSyscalls implements `libtock_platform::Syscalls`.
pub struct TockSyscalls;

#[cfg(target_arch = "arm")]
mod syscalls_impl_arm;
#[cfg(target_arch = "riscv32")]
mod syscalls_impl_riscv;
