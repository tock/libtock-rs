#![no_std]

mod error_code;
mod syscalls;

pub use error_code::ErrorCode;
pub use syscalls::{MemopNoArg, MemopWithArg, Syscalls};
