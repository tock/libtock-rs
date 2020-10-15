// The most minimal libtock_core example possible. This file primarily exists
// for code size measurement, as this should create the smallest-possible
// libtock_core app.

#![no_std]

libtock_core::stack_size! {0x400}

// If you don't *use* anything from libtock_core directly, cargo will not link
// it into the executable. However, we still need the runtime and lang items.
// Therefore a libtock_core app that doesn't directly mention anything in
// libtock_core needs to explicitly declare its dependency on libtock_core as
// follows.
extern crate libtock_core;

fn main() {}
