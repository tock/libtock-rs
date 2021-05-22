//! Defines constants shared between multiple `libtock-rs` crates.

pub mod syscall_class {
    pub const SUBSCRIBE: usize = 1;
    pub const COMMAND: usize = 2;
    pub const RW_ALLOW: usize = 3;
    pub const RO_ALLOW: usize = 4;
    pub const MEMOP: usize = 5;
    pub const EXIT: usize = 6;
}

pub mod yield_id {
    pub const NO_WAIT: u32 = 0;
    pub const WAIT: u32 = 1;
}
