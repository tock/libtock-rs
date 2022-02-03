#![cfg_attr(not(test), no_std)]
#![warn(unsafe_op_in_unsafe_fn)]

pub mod allow_ro;
pub mod allow_rw;
pub mod command_return;
mod constants;
mod default_config;
mod error_code;
pub mod exit_on_drop;
mod raw_syscalls;
mod register;
pub mod return_variant;
pub mod share;
pub mod subscribe;
mod syscalls;
mod syscalls_impl;
mod termination;
mod yield_types;

pub use allow_ro::AllowRo;
pub use allow_rw::AllowRw;
pub use command_return::CommandReturn;
pub use constants::{exit_id, syscall_class, yield_id};
pub use default_config::DefaultConfig;
pub use error_code::ErrorCode;
pub use raw_syscalls::RawSyscalls;
pub use register::Register;
pub use return_variant::ReturnVariant;
pub use subscribe::{Subscribe, Upcall};
pub use syscalls::Syscalls;
pub use termination::Termination;
pub use yield_types::YieldNoWaitReturn;

#[cfg(test)]
mod command_return_tests;

#[cfg(test)]
mod error_code_tests;
