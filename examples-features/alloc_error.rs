// Triggers the out-of-memory handler. Should print an error message.

#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::vec::Vec;
use core::alloc::Layout;
use libtock::println;
use libtock::result::TockResult;
use libtock::syscalls;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x800] = [0; 0x800];

#[libtock::main]
fn main() -> TockResult<()> {
    let mut vec = Vec::new();
    loop {
        vec.push(0);
    }
}

#[alloc_error_handler]
unsafe fn alloc_error_handler(_: Layout) -> ! {
    if let Ok(drivers) = libtock::retrieve_drivers() {
        drivers.console.create_console();
        println!("alloc_error_handler called");
    }
    loop {
        syscalls::raw::yieldk();
    }
}
