#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]
#![cfg_attr(any(target_arch = "arm", target_arch = "riscv32"), no_std)]

#[cfg(feature = "alloc")]
mod alloc;

pub mod adc;
pub mod ble_composer;
pub mod ble_parser;
pub mod buttons;
pub mod console;
pub mod debug;
pub mod drivers;
pub mod electronics;
pub mod futures;
pub mod gpio;
pub mod leds;
pub mod result;
pub mod rng;
pub mod sensors;
pub mod simple_ble;
pub mod temperature;
pub mod timer;

pub use drivers::retrieve_drivers;
pub use libtock_codegen::main;
pub use libtock_core::*;
