#![forbid(unsafe_code)]
#![no_std]

pub use libtock_platform as platform;
pub use libtock_runtime as runtime;

pub mod low_level_debug {
    use libtock_low_level_debug as lldb;
    pub type LowLevelDebug = lldb::LowLevelDebug<super::runtime::TockSyscalls>;
    pub use lldb::AlertCode;
}
