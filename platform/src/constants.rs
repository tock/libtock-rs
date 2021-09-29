//! Defines constants shared between multiple `libtock-rs` crates.

pub mod exit_id {
    pub const TERMINATE: u32 = 0;
    pub const RESTART: u32 = 1;
}

pub mod syscall_class {
    pub const SUBSCRIBE: usize = 1;
    pub const COMMAND: usize = 2;
    pub const ALLOW_RW: usize = 3;
    pub const ALLOW_RO: usize = 4;
    pub const MEMOP: usize = 5;
    pub const EXIT: usize = 6;
}

pub mod yield_id {
    pub const NO_WAIT: u32 = 0;
    pub const WAIT: u32 = 1;
}
