//! `libtock_unittest` provides testing tools needed by `libtock-rs`'s own unit
//! tests as well as unit tests of code that uses `libtock-rs`.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod command_return;
mod driver;
mod expected_syscall;
mod kernel;
mod kernel_data;
mod syscall_log;
pub mod upcall;

/// `fake` contains fake implementations of Tock kernel components. Fake
/// components emulate the behavior of the real Tock kernel components, but in
/// the unit test environment. They generally have additional testing features,
/// such as error injection functionality.
///
/// These components are exposed under the `fake` module because otherwise their
/// names would collide with the corresponding drivers (e.g. the fake Console
/// would collide with the Console driver in unit tests). Tests should generally
/// `use libtock_unittest::fake` and refer to the type with the `fake::` prefix
/// (e.g. `fake::Console`).
pub mod fake {
    pub use crate::driver::Driver;
    pub use crate::kernel::Kernel;
}

pub use expected_syscall::ExpectedSyscall;
pub use syscall_log::SyscallLogEntry;
