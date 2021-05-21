#![no_std]
#![warn(unsafe_op_in_unsafe_fn)]

mod async_traits;
mod command_return;
mod constants;
mod error_code;
mod raw_syscalls;
mod register;
pub mod return_variant;
mod syscalls;
mod syscalls_impl;
mod yield_types;

pub use async_traits::{CallbackContext, FreeCallback, Locator, MethodCallback};
pub use command_return::CommandReturn;
pub use constants::{syscall_class, yield_id};
pub use error_code::ErrorCode;
pub use raw_syscalls::RawSyscalls;
pub use register::Register;
pub use return_variant::ReturnVariant;
pub use syscalls::Syscalls;
pub use yield_types::YieldNoWaitReturn;

#[cfg(test)]
mod command_return_tests;

#[cfg(test)]
mod error_code_tests;
