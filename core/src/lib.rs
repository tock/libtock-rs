#![feature(asm, lang_items, naked_functions)]
#![cfg_attr(any(target_arch = "arm", target_arch = "riscv32"), no_std)]

mod entry_point;
#[cfg(any(target_arch = "arm", target_arch = "riscv32"))]
mod lang_items;

pub mod callback;
pub mod debug;
pub mod memop;
pub mod result;
pub mod shared_memory;
pub mod syscalls;
pub mod unwind_symbols;
