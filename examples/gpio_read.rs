#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::gpio::{GpioPinUnitialized, InputMode};
use libtock::timer;
use libtock::timer::Duration;

// example works on p0.03
fn main() {
    let mut console = Console::new();
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_read(None, InputMode::PullDown).unwrap();

    loop {
        if pin.read() {
            writeln!(console, "true").unwrap();
        } else {
            writeln!(console, "false").unwrap();
        }
        timer::sleep(Duration::from_ms(500));
    }
}
