// Triggers the panic handler. Should print an error message.

#![no_std]

use core::fmt::Write;
use core::panic::PanicInfo;
use libtock::result::TockResult;
use libtock::syscalls;

#[libtock::main]
async fn main() -> TockResult<()> {
    panic!("Bye world!");
}

#[panic_handler]
unsafe fn panic_handler(_info: &PanicInfo) -> ! {
    if let Ok(drivers) = libtock::retrieve_drivers() {
        let mut console = drivers.console.create_console();
        let _ = writeln!(console, "panic_handler called");
    }
    loop {
        syscalls::raw::yieldk();
    }
}
