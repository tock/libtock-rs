//! `fake` contains fake implementations of Tock kernel components. Fake
//! components emulate the behavior of the real Tock kernel components, but in
//! the unit test environment. They generally have additional testing features,
//! such as error injection functionality.
//!
//! These components are exposed under the `fake` module because otherwise their
//! names would collide with the corresponding drivers (e.g. the fake Console
//! would collide with the Console driver in unit tests). Tests should generally
//! `use libtock_unittest::fake` and refer to the type with the `fake::` prefix
//! (e.g. `fake::Console`).

mod driver;
mod kernel;
mod syscalls;

pub use driver::Driver;
pub use kernel::Kernel;
pub use syscalls::Syscalls;

#[cfg(test)]
mod kernel_tests;
