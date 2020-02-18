#![feature(asm, lang_items, naked_functions)]
#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]
#![cfg_attr(any(target_arch = "arm", target_arch = "riscv32"), no_std)]

#[cfg(feature = "alloc")]
mod alloc;
mod entry_point;
#[cfg(any(target_arch = "arm", target_arch = "riscv32"))]
mod lang_items;

pub mod adc;
pub mod ble_composer;
pub mod ble_parser;
pub mod buttons;
pub mod callback;
pub mod console;
pub mod debug;
pub mod drivers;
pub mod electronics;
pub mod futures;
pub mod gpio;
pub mod leds;
pub mod memop;
pub mod result;
pub mod rng;
pub mod sensors;
pub mod shared_memory;
pub mod simple_ble;
pub mod syscalls;
pub mod temperature;
pub mod timer;
pub mod unwind_symbols;

pub use drivers::retrieve_drivers;
pub use libtock_codegen::main;
