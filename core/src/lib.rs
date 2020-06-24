#![feature(lang_items, llvm_asm, naked_functions)]
#![cfg_attr(any(target_arch = "arm", target_arch = "riscv32"), no_std)]
#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]

#[cfg(feature = "alloc")]
mod alloc;
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
