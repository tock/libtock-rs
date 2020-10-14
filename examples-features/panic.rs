// Triggers the panic handler. Should print an error message.

#![no_std]

use core::panic::PanicInfo;
use libtock::println;
use libtock::result::TockResult;
use libtock::syscalls;

#[libtock::main]
async fn main() -> TockResult<()> {
    panic!("Bye world!");
}

#[panic_handler]
unsafe fn panic_handler(_info: &PanicInfo) -> ! {
    if let Ok(drivers) = libtock::retrieve_drivers() {
        drivers.console.create_console();
        println!("panic_handler called");
    }
    loop {
        syscalls::raw::yieldk();
    }
}
