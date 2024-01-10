#![no_std]
#![no_main]

use core::fmt::Write;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::console::Console;
use libtock::rng::Rng;
use libtock::runtime::{set_main, stack_size};

stack_size! {0x300}
set_main! {main}

struct Randomness<'a, const N: usize>(&'a [u8; N]);

impl<'a, const N: usize> core::fmt::Display for Randomness<'a, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut bytes = self.0.iter();
        while let Some(&byte) = bytes.next() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

fn main() {
    if let Err(e) = Rng::exists() {
        writeln!(Console::writer(), "RNG DRIVER ERROR: {e:?}").unwrap();
        return;
    }

    let mut console_writer = Console::writer();
    let mut buffer: [u8; 32] = Default::default();
    let n: u32 = 32;

    loop {
        match Rng::get_bytes_sync(&mut buffer, n) {
            Ok(()) => {
                let _ = writeln!(console_writer, "Randomness: {}", Randomness(&buffer));
            }
            Err(e) => {
                let _ = writeln!(console_writer, "Error while getting bytes {e:?}");
            }
        }
        let _ = Alarm::sleep_for(Milliseconds(2000));
    }
}
