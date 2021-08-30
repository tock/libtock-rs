//! `libtock_unittest` provides testing tools needed by `libtock-rs`'s own unit
//! tests as well as unit tests of code that uses `libtock-rs`.

#![deny(unsafe_op_in_unsafe_fn)]

mod allow_db;
pub mod command_return;
mod expected_syscall;
pub mod fake;
mod kernel_data;
mod syscall_log;
pub mod upcall;

pub use allow_db::{RoAllowBuffer, RwAllowBuffer};
pub use expected_syscall::ExpectedSyscall;
pub use syscall_log::SyscallLogEntry;

#[cfg(test)]
mod allow_db_test;
