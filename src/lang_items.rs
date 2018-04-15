//! Lang item required to make the normal `main` work in applications
//!
//! This is how the `start` lang item works:
//! When `rustc` compiles a binary crate, it creates a `main` function that looks
//! like this:
//!
//! ```
//! #[export_name = "main"]
//! pub extern "C" fn rustc_main(argc: isize, argv: *const *const u8) -> isize {
//!     start(main, argc, argv)
//! }
//! ```
//!
//! Where `start` is this function and `main` is the binary crate's `main`
//! function.
//!
//! The final piece is that the entry point of our program, _start, has to call
//! `rustc_main`. That's covered by the `_start` function in the root of this
//! crate.

use led;
use timer;
use timer::Duration;

#[lang = "start"]
extern "C" fn start<T>(main: fn() -> T, _argc: isize, _argv: *const *const u8) -> isize
where
    T: Termination,
{
    main();

    0
}

#[lang = "termination"]
pub trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        0
    }
}

#[lang = "eh_personality"]
fn eh_personality() {
    cycle_leds();
}

#[lang = "panic_fmt"]
fn panic_fmt() {
    flash_all_leds();
}

fn cycle_leds() {
    for led in led::all().cycle() {
        led.on();
        timer::sleep(Duration::from_ms(100));
        led.off();
    }
}

fn flash_all_leds() {
    loop {
        for led in led::all() {
            led.on();
        }
        timer::sleep(Duration::from_ms(100));
        for led in led::all() {
            led.off();
        }
        timer::sleep(Duration::from_ms(100));
    }
}
