#![feature(asm, lang_items)]
#![no_std]

pub mod syscalls;
pub mod timer;
pub mod led;

mod lang_items;

use core::ptr;

/// Tock programs' entry point
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    extern "C" {
        // NOTE `rustc` forces this signature on us. See `src/lang_items.rs`
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }

    // arguments are not used in Tock applications
    unsafe {
        main(0, ptr::null());
    }

    loop {
        ::syscalls::yieldk();
    }
}
