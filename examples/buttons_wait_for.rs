//! An extremely simple libtock-rs example. Register button events.
#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::buttons::{ButtonListener, Buttons};
use libtock::console::Console;
use libtock::leds::Leds;
use libtock::runtime::{set_main, stack_size};
use libtock_platform::share;
// use libtock_runtime::TockSyscalls;

set_main! {main}
stack_size! {0x1000}

fn main() {
    writeln!(Console::writer(), "main!").unwrap();
    let listener = ButtonListener(|button, _state| {
        let _ = Leds::toggle(button);
        // writeln!(Console::writer(), "button {:?}: {:?}", button, state).unwrap();
    });
    if let Ok(buttons_count) = Buttons::count() {
        writeln!(Console::writer(), "button count: {}", buttons_count).unwrap();

        share::scope(|subscribe| {
            Buttons::register_listener(&listener, subscribe).unwrap();
            // Enable interrupts for each button press.
            for i in 0..buttons_count {
                Buttons::enable_interrupts(i).unwrap();
            }

            // Wait for buttons to be pressed.
            loop {
                let driver_number: u32 = 0x3;
                let subscribe_number: u32 = 0;
                let (status, state) = Buttons::wait_for_button(driver_number, subscribe_number);
                writeln!(
                    Console::writer(),
                    "Button pressed (yield_wait_for), status: {:?}, state: {:?}",
                    status,
                    state
                )
                .unwrap();
            }
        });
    }
}
