#![no_std]
#![no_main]

use core::fmt::Write;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::rng::RngListener;
use libtock::{console::Console, rng::Rng};
use libtock_platform::{share, Syscalls};
use libtock_runtime::{set_main, stack_size, TockSyscalls};

stack_size! {0x300}
set_main! {main}

fn main() {
    if let Err(e) = Rng::exists() {
        writeln!(Console::writer(), "RNG DRIVER ERROR: {e:?}").unwrap();
        return;
    }

    let mut console_writer = Console::writer();
    let rng_listener = RngListener(|_| write!(Console::writer(), "Randomness: ").unwrap());
    let mut buffer: [u8; 32] = Default::default();
    let n: u32 = 32;

    loop {
        share::scope(|allow_rw| {
            Rng::allow_buffer(&mut buffer, allow_rw).unwrap();

            share::scope(|subscribe| {
                Rng::register_listener(&rng_listener, subscribe).unwrap();

                Rng::get_bytes_async(n).unwrap();
                TockSyscalls::yield_wait();
            });
        });

        buffer.iter().for_each(|&byte| {
            let _ = write!(console_writer, "{byte:02x}");
        });
        let _ = writeln!(console_writer, "");

        let _ = Alarm::sleep_for(Milliseconds(2000));
    }
}
