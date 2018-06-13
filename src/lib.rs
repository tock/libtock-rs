#![feature(asm, alloc, allocator_api, lang_items, naked_functions, panic_implementation)]
#![no_std]

extern crate alloc;

mod callback;

pub mod ble_parser;
pub mod buttons;
pub mod console;
pub mod debug;
pub mod electronics;
pub mod fmt;
pub mod gpio;
pub mod ipc;
pub mod led;
pub mod result;
pub mod sensors;
pub mod shared_memory;
pub mod simple_ble;
pub mod temperature;
pub mod timer;

#[cfg(target_arch = "arm")]
pub mod entry_point;
#[cfg(target_arch = "arm")]
mod lang_items;
#[cfg(target_arch = "arm")]
pub mod syscalls;
#[cfg(not(target_arch = "arm"))]
#[path = "syscalls_mock.rs"]
mod syscalls;

#[cfg(target_arch = "arm")]
#[global_allocator]
static ALLOCATOR: entry_point::TockAllocator = entry_point::TockAllocator;
