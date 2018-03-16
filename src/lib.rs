#![feature(asm, alloc, allocator_api, compiler_builtins_lib, global_allocator, lang_items,
           naked_functions)]
#![no_std]

extern crate alloc;
extern crate compiler_builtins;

mod callback;

pub mod ble_parser;
pub mod buttons;
pub mod console;
pub mod debug;
pub mod electronics;
pub mod fmt;
pub mod gpio;
pub mod ipc;
pub mod ipc_cs;
pub mod led;
pub mod result;
pub mod sensors;
pub mod simple_ble;
pub mod temperature;
pub mod timer;
pub mod shared_memory;

#[cfg(target_os = "tock")]
pub mod entry_point;
#[cfg(target_os = "tock")]
mod lang_items;
#[cfg(target_os = "tock")]
pub mod syscalls;
#[cfg(not(target_os = "tock"))]
#[path = "syscalls_mock.rs"]
mod syscalls;

#[cfg(target_os = "tock")]
#[global_allocator]
static ALLOCATOR: entry_point::StackOwnedHeap = entry_point::StackOwnedHeap;
