//! An extremely simple libtock-rs example.
//! Just prints out some debug messages
//! using the debug macros, then terminates.

#![no_main]
#![no_std]
use libtock::print_macros::{dbg, print, println};
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x100}

#[derive(Debug)]
struct HelloWorld;

fn main() {
    dbg!(HelloWorld);
    let i: u32 = dbg!(0xdeadc0deu32);
    print!("Hello");
    println!(" again! {}", i);
}
