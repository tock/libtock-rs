#![no_std]

use libtock_core::result::CommandError;

fn main() -> Result<(), CommandError> {
    panic!("Bye world!");
}

#[panic_handler]
unsafe fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
