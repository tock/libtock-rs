//! A simple GPIO example for getting GPIO interrupts.
//!
//! This will configure GPIO 0 to be a rising-edge triggered interrupt and print
//! a message when the interrupt is triggered.

#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::gpio;
use libtock::gpio::Gpio;
use libtock::runtime::{set_main, stack_size};
use libtock_platform::{share, Syscalls};
use libtock_runtime::TockSyscalls;

set_main! {main}
stack_size! {0x1000}

fn main() {
    let listener = gpio::GpioInterruptListener(|gpio_index, state| {
        writeln!(Console::writer(), "GPIO[{}]: {:?}", gpio_index, state).unwrap();
    });

    if !Gpio::count().is_ok_and(|c| c > 0) {
        writeln!(Console::writer(), "No GPIO pins on this board.").unwrap();
        return;
    }

    // Configure pin 0 as an input and enable rising interrupts
    let pin = Gpio::get_pin(0).unwrap();
    let input_pin = pin.make_input::<gpio::PullNone>().unwrap();
    let _ = input_pin.enable_interrupts(gpio::PinInterruptEdge::Rising);

    // Wait for callbacks.
    share::scope(|subscribe| {
        Gpio::register_listener(&listener, subscribe).unwrap();

        loop {
            TockSyscalls::yield_wait();
        }
    });
}
