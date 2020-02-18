// Triggers the out-of-memory handler. Should make all LEDs cycle.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use libtock::result::TockResult;

#[libtock::main]
fn main() -> TockResult<()> {
    let mut vec = Vec::new();
    loop {
        vec.push(0);
    }
}
