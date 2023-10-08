#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::alarm::Alarm;
use libtock::console::Console;
use libtock::rng::Rng;
use libtock::runtime::{set_main, stack_size};
use libtock_alarm::Milliseconds;

set_main! {main}
stack_size! {0x300}

struct Randomness<'a, const N: usize>(&'a [u8; N]);

impl<'a, const N: usize> core::fmt::Display for Randomness<'a, N> {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut console_writer = Console::writer();
        let mut bytes = self.0.iter();
        while let Some(&byte) = bytes.next() {
            write!(console_writer, "{byte:02x}")?;
        }
        Ok(())
    }
}

fn main() {
    if let Err(e) = Rng::exists() {
        writeln!(
            Console::writer(),
            "RNG DRIVER DOESN'T EXIST\nERROR: {e:?}\n"
        )
        .unwrap();
        return;
    }

    let mut buffer: [u8; 32] = Default::default();
    let n: u32 = 32;
    let mut console_writer = Console::writer();
    loop {
        match Rng::get_bytes_sync(&mut buffer, n) {
            Ok(()) => {
                write!(console_writer, "Randomness: {}\n", Randomness(&buffer)).unwrap();
            }
            Err(e) => writeln!(console_writer, "Error while getting bytes {e:?}\n").unwrap(),
        }
        Alarm::sleep_for(Milliseconds(2000)).unwrap();
    }
}
