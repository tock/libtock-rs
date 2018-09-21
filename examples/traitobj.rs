#![feature(prelude_import)]
#![no_std]
#![feature(alloc)]
#![feature(compiler_builtins_lib)]
#[prelude_import]
#[macro_use]
extern crate alloc;
extern crate tock;
use core::mem;
use core::ptr;
use tock::console::Console;
use tock::debug;
use tock::gpio::{GpioPinUnitialized, InputMode};
use tock::timer;
use tock::timer::Duration;
pub trait MyTrait {
    fn print_something(&self);
}

impl MyTrait for usize {
    fn print_something(&self) {
        debug::print_as_hex(*self);
    }
}

impl MyTrait for u64 {
    fn print_something(&self) {
        debug::print_as_hex(*self as usize);
    }
}
fn dispatch_printing(blub: &MyTrait) {
    blub.print_something();
}
fn dynamic_dispatch_works() {
    let my_usize = 5usize;
    let my_u64 = 7u64;

    // Inhibit optimization
    let pin = GpioPinUnitialized::new(0);
    let pin = pin.open_for_read(None, InputMode::PullDown).unwrap();

    // Some debug info
    unsafe {
        let vtable_location = mem::transmute::<_, (usize, usize)>(&my_usize as &MyTrait).1;
        debug::print_as_hex(vtable_location as usize);

        let first_vtable_entry = ptr::read_volatile(vtable_location as *const usize);
        debug::print_as_hex(first_vtable_entry as usize);

        // useful for finding offsets via objdump
        let destructor = ptr::read_volatile(first_vtable_entry as *const usize);
        debug::print_as_hex(destructor as usize);
    }

    let mut w: &MyTrait;
    if !pin.read() {
        w = &my_u64;
    } else {
        w = &my_usize;
    }
    dispatch_printing(w);
    if pin.read() {
        w = &my_u64;
    } else {
        w = &my_usize;
    }
    dispatch_printing(w);
}

fn format_works() {
    let mut console = Console::new();
    console.write(&format!("Test Zahl {} String {}\n", 3, "Kebes"));
}
fn main() {
    dynamic_dispatch_works();
    format_works();
    for _ in 0.. {
        timer::sleep(Duration::from_ms(500))
    }
}
