//! Runtime components related to process startup.

use crate::TockSyscalls;
use libtock_platform::RawSyscalls;

/// `set_main!` is used to tell `libtock_runtime` where the process binary's
/// `main` function is. The process binary's `main` function must have the
/// signature `FnOnce() -> T`, where T is some concrete type that implements
/// `libtock_platform::Termination`.
///
/// # Example
/// ```
/// libtock_runtime::set_main!{main};
///
/// fn main() -> () { /* Omitted */ }
/// ```
// set_main! generates a function called `libtock_unsafe_main`, which is called
// by `rust_start`. The function has `unsafe` in its name because implementing
// it is `unsafe` (it *must* have the signature `libtock_unsafe_main() -> !`),
// but there is no way to enforce the use of `unsafe` through the type system.
// This function calls the client-provided function, which enforces its type
// signature.
#[macro_export]
macro_rules! set_main {
    {$name:ident} => {
        #[no_mangle]
        fn libtock_unsafe_main() -> ! {
            use libtock_runtime::TockSyscalls;
            let res = $name();
            #[allow(unreachable_code)] // so that fn main() -> ! does not produce a warning.
            libtock_platform::Termination::complete::<TockSyscalls>(res)
        }
    }
}

/// Executables must specify their stack size by using the `stack_size!` macro.
/// It takes a single argument, the desired stack size in bytes. Example:
/// ```
/// stack_size!{0x400}
/// ```
// stack_size works by putting a symbol equal to the size of the stack in the
// .stack_buffer section. The linker script uses the .stack_buffer section to
// size the stack. flash.sh looks for the symbol by name (hence #[no_mangle]) to
// determine the size of the stack to pass to elf2tab.
#[macro_export]
macro_rules! stack_size {
    {$size:expr} => {
        #[no_mangle]
        #[link_section = ".stack_buffer"]
        pub static mut STACK_MEMORY: [u8; $size] = [0; $size];
    }
}

// rust_start is the first Rust code to execute in the process. It is called
// from start, which is written directly in assembly.
#[no_mangle]
extern "C" fn rust_start() -> ! {
    // TODO: Call memop() to inform the kernel of the stack and size +
    // locations (for debugging).
    // Also, perhaps we should support calling a heap initialization
    // function?

    extern "Rust" {
        // This function is created by the set_main!() macro.
        fn libtock_unsafe_main() -> !;
    }
    // TODO: Provide mechanism for dynamic heap size
    let app_heap_size: usize = 2048;
    unsafe {
        // TODO: Replace with non-raw syscalls once memop
        // implemented.
        let _app_heap_start = TockSyscalls::syscall1::<5>([0u32.into()]);
        //let _app_heap_start = memop::get_brk();
        // Tell the kernel the new app heap break.
        TockSyscalls::syscall2::<5>([1u32.into(), app_heap_size.into()]);
        //memop::increment_brk(app_heap_size);

        libtock_unsafe_main();
    }
}
