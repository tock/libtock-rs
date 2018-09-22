#![no_std]

extern crate tock;

use tock::console::Console;
use tock::gpio::{GpioPinUnitialized, InputMode};
use tock::timer;
use tock::timer::Duration;

// example works on p0.03
fn main() {
    let mut console = Console::new();
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_read(None, InputMode::PullDown).unwrap();

    loop {
        if pin.read() {
            console.write("true\n");
        } else {
            console.write("false\n");
        }
        timer::sleep(Duration::from_ms(500));
    }
}
