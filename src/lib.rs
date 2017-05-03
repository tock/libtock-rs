#![feature(asm,lang_items)]
#![no_std]

pub mod syscalls;
pub mod timer;
pub mod led;

#[macro_export]
macro_rules! tock_main {
    ($init:expr) => {
        #[no_mangle]
        #[allow(unreachable_code)]
        pub extern "C" fn _start() -> ! {
            $init
            loop {
                ::tock::syscalls::yieldk();
            }
        }
    }
}

#[lang="eh_personality"]
pub extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind() {
    loop {}
}
