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

mod buttons;
mod console;
mod gpio;
mod kernel;
mod leds;
mod low_level_debug;
mod syscall_driver;
mod syscalls;

pub use buttons::Buttons;
pub use console::Console;
pub use gpio::{Gpio, GpioMode, InterruptEdge, PullMode};
pub use kernel::Kernel;
pub use leds::Leds;
pub use low_level_debug::{LowLevelDebug, Message};
pub use syscall_driver::SyscallDriver;
pub use syscalls::Syscalls;

#[cfg(test)]
mod kernel_tests;
