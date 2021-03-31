#![feature(min_const_generics)] // TODO: Remove when toolchain updated.
#![no_std]

mod async_traits;
mod command_return;
mod error_code;
mod raw_syscalls;
pub mod return_variant;
mod syscalls;
mod syscalls_impl;
mod yield_types;

pub use async_traits::{CallbackContext, FreeCallback, Locator, MethodCallback};
pub use command_return::CommandReturn;
pub use error_code::ErrorCode;
pub use raw_syscalls::RawSyscalls;
pub use return_variant::ReturnVariant;
pub use syscalls::Syscalls;
pub use yield_types::YieldNoWaitReturn;

#[cfg(test)]
mod command_return_tests;
