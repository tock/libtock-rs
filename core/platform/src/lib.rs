#![no_std]

// TODO: Implement this crate, which will be done piece-by-piece. Platform will
// include:
//   1. The Allowed and AllowedSlice abstractions for sharing memory with the
//      kernel
//   2. The PlatformApi trait and Platform implementation.
//   3. A system call trait so that Platform works in both real Tock apps and
//      unit test environments. [DONE]

mod allows;
mod async_traits;
mod error_code;
mod platform_api;
mod return_code;
mod syscalls;

pub use allows::{AllowReadable, Allowed, AllowedSlice, OutOfBounds};
pub use async_traits::{AsyncCall, Callback, CallbackContext, StaticCallback};
pub use error_code::ErrorCode;
pub use platform_api::PlatformApi;
pub use return_code::ReturnCode;
pub use syscalls::{MemopNoArg, MemopWithArg, Syscalls};
