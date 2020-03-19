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

use crate::syscalls;

#[lang = "start"]
extern "C" fn start<T>(main: fn() -> T, _argc: isize, _argv: *const *const u8) -> bool
where
    T: Termination,
{
    main().check_result();
    true // Need to return anything sized. Otherwise, a linker error pops up. (See https://github.com/tock/libtock-rs/issues/138)
}

#[lang = "termination"]
pub trait Termination {
    fn check_result(self);
}

impl Termination for () {
    fn check_result(self) {}
}

impl<S, T> Termination for Result<S, T> {
    fn check_result(self) {
        if self.is_err() {
            unsafe { report_panic() };
        }
    }
}

#[cfg(not(feature = "custom_panic_handler"))]
#[panic_handler]
unsafe fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    report_panic()
}

unsafe fn report_panic() -> ! {
    // Signal a panic using the LowLevelDebug capsule (if available).
    let _ = syscalls::command1_insecure(8, 1, 1);

    loop {
        syscalls::raw::yieldk();
    }
}
