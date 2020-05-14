// The most minimal libtock_core example possible. This file primarily exists
// for code size measurement, as this should create the smallest-possible
// libtock_core app.

#![no_std]

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x400] = [0; 0x400];

// If you don't *use* anything from libtock_core directly, cargo will not link
// it into the executable. However, we still need the runtime and lang items.
// Therefore a libtock_core app that doesn't directly mention anything in
// libtock_core needs to explicitly declare its dependency on libtock_core as
// follows.
extern crate libtock_core;

fn main() {}
