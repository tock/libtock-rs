// The most minimal libtock-core example possible. This file primarily exists
// for code size measurement, as this should create the smallest-possible
// libtock-core app.

#![no_std]

// If you don't *use* anything from libtock-core directly, cargo will not link
// it into the executable. However, we still need the runtime and lang items.
// Therefore a libtock-core app that doesn't directly mention anything in
// libtock-core needs to explicitly declare its dependency on libtock-core as
// follows.
extern crate libtock_core;

fn main() {}
