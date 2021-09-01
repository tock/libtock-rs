#![feature(asm, lang_items, llvm_asm, naked_functions)]
#![cfg_attr(any(target_arch = "arm", target_arch = "riscv32"), no_std)]
#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]

#[cfg(feature = "alloc")]
mod alloc;
#[cfg(any(target_arch = "arm", target_arch = "riscv32"))]
mod lang_items;

#[cfg(feature = "alloc_init")]
extern "Rust" {
    fn libtock_alloc_init(app_heap_start: usize, app_heap_size: usize);
}

pub mod callback;
pub mod debug;
mod entry_point;
pub mod memop;
pub mod result;
pub mod shared_memory;
pub mod stack_size;
pub mod syscalls;
pub mod unwind_symbols;
