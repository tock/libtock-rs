#[cfg_attr(target_arch = "arm", path = "platform_arm.rs")]
#[cfg_attr(target_arch = "riscv32", path = "platform_riscv32.rs")]
mod platform;

pub use platform::*;
