#![no_std]

// TODO: Implement this crate, which will be done piece-by-piece. Platform will
// include:
//   1. The Allowed and AllowedSlice abstractions for sharing memory with the
//      kernel
//   2. The PlatformApi trait and Platform implementation.
//   3. A system call trait so that Platform works in both real Tock apps and
//      unit test environments. [DONE]

mod allows;
mod error_code;
mod syscall_types;
mod syscalls;

pub use allows::{AllowReadable, Allowed};
pub use error_code::ErrorCode;
pub use syscall_types::{OneArgMemop, ReturnType, YieldType, ZeroArgMemop};
pub use syscalls::Syscalls;
