#![feature(asm,alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::fmt::Write;
use tock::console::Console;
use tock::sensors::Ninedof;

fn main() {
    let mut console = Console::new();
    let mut ninedof = unsafe { Ninedof::new() };
    loop {
        let accel = ninedof.read_acceleration();
        write!(&mut console, "X: {}\n", accel.x).unwrap();
        write!(&mut console, "Y: {}\n", accel.y).unwrap();
        write!(&mut console, "Z: {}\n", accel.z).unwrap();
        tock::timer::delay_ms(500);
    }
}

