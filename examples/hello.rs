#![feature(asm,alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::fmt::Write;
use tock::console::Console;

fn main() {
    let mut console = Console::new();
    for i in 0.. {
        write!(&mut console, "Hello world! {}\n", i).unwrap();
        tock::timer::delay_ms(500);
    }
}

