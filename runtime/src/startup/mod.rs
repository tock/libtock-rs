//! Runtime components related to process startup.

use crate::TockSyscalls;
use libtock_platform::{Syscalls, Termination};

/// Program startup function.
///
/// The kernel jumps here with an initial, small, stack allocated for
/// execution. [start] sets up the environment necesarry to invoke the
/// first rust function: [rust_start].
///
/// # Arguments:
///
/// All arguments are word-sized values stored in the
/// calling-convention registers.
///
/// - app_start
/// - mem_start
/// - memory_len
/// - app_heap_break
#[cfg(target_arch = "arm")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() {
    core::arch::naked_asm!(
        // First, verify the process binary was loaded at the correct address. The
        // check is performed by comparing the program counter at the start to the
        // address of `start`, which is stored in rt_header.
        "mov r4, pc",       // r4 = address of .start + 4 (Thumb bit unset)
        "mov r5, r0",       // Save rt_header; we use r0 for syscalls
        "ldr r0, [r5, #0]", // r0 = rt_header.start
        "adds r0, #4",      // r0 = rt_header.start + 4
        "cmp r0, r4",       // Skip error handling if pc correct
        "beq 1f",
        // If the beq on the previous line did not jump, then the binary is not at
        // the correct location. Report the error via LowLevelDebug then exit.
        "movs r0, #8", // LowLevelDebug driver number
        "movs r1, #1", // Command: print alert code
        "movs r2, #2", // Alert code 2 (incorrect location
        "svc 2",       // Execute `command`
        "movs r0, #0", // Operation: exit-terminate
        "movs r1, #1", // Completion code: FAIL
        "svc 6",       // Execute `exit`
        //
        // fn set_brk()
        // set brk to rt_header's initial break value
        "1:",
        "movs r0, #0",      // operation: set break
        "ldr r1, [r5, #4]", // rt_header`s initial process break
        "svc 5",            // call `memop`
        // Set the stack pointer
        "ldr r0, [r5, #8]", // r0 = rt_header._stack_top
        "mov sp, r0",
        // Copy .data into place
        "ldr r0, [r5, #12]", // remaining = rt_header.data_size
        "cmp r0, #0",        // Jump to zero_bss if remaining == 0
        "beq 3f",
        "ldr r1, [r5, #16]", // src = rt_header.data_flash_start
        "ldr r2, [r5, #20]", // dest = rt_header.data_ram_start
        //
        // fn data_loop_body()
        "2:",
        "ldr r3, [r1]", // r3 = *src
        "str r3, [r2]", // *(dest) = r3
        "subs r0, #4",  // remaining -= 4
        "adds r1, #4",  // src += 4
        "adds r2, #4",  // dest += 4
        "cmp r0, #0",
        "bne 2b", // Iterate again if remaining != 0
        //
        // fn zero_bss()
        "3:",
        "ldr r0, [r5, #24]", // remaining = rt_header.bss_size
        "cmp r0, #0",        // Jump to call_rust_start if remaining == 0
        "beq 5f",
        "ldr r1, [r5, #28]", // dest = rt_header.bss_start
        "movs r2, #0",       // r2 = 0
        //
        // fn bss_loop_body()
        "4:",
        "strb r2, [r1]", // *(dest) = r2 = 0
        "subs r0, #1",   // remaining -= 1
        "adds r1, #1",   // dest += 1
        "cmp r0, #0",
        "bne 4b", // Iterate again if remaining != 0
        //
        // fn call_rust_start()
        "5:",
        "bl {trampoline}",
        trampoline = sym rust_start,
    );
}
#[cfg(target_arch = "riscv32")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn start() {
    core::arch::naked_asm!(
    // First, verify the process binary was loaded at the correct address. The
    // check is performed by comparing the program counter at the start to the
    // address of `start`, which is stored in rt_header. */
    "auipc s0, 0",           // s0 = pc
    "mv a5, a0",             // Save rt_header so syscalls don't overwrite it
    "lw s1, 0(a5)",          // s1 = rt_header.start
    "beq s0, s1, 1f",        // Skip error handling code if pc is correct
    // If the beq on the previous line did not jump, then the binary is not at
    // the correct location. Report the error via LowLevelDebug then exit.
    "li a0, 8",  // LowLevelDebug driver number
    "li a1, 1",  // Command: Print alert code
    "li a2, 2",  // Alert code 2 (incorrect location)
    "li a4, 2",  // `command` class
    "ecall",
    "li a0, 0",  // exit-terminate
    "li a1, 1",  // Completion code: FAIL
    "li a4, 6",  // `exit` class
    "ecall",

    // fn set_brk()
    "1:",
    // memop(): set brk to rt_header's initial break value
    "li a0, 0",      // operation: set break
    "lw a1, 4(a5)",  // rt_header's initial process break
    "li a4, 5",      // `memop` class
    "ecall",

    // Set the stack pointer
    "lw sp, 8(a5)",  // sp = rt_header._stack_top

    // Copy .data into place.
    "lw a0, 12(a5)",              // remaining = rt_header.data_size
    "beqz a0, 3f",                // Jump to zero_bss if remaining is zero
    "lw a1, 16(a5)",              // src = rt_header.data_flash_start
    "lw a2, 20(a5)",              // dest = rt_header.data_ram_start
    // fn data_loop_body:
    "2:",
    "lw a3, 0(a1)",               // a3 = *src
    "sw a3, 0(a2)",               // *dest = a3
    "addi a0, a0, -4",            // remaining -= 4
    "addi a1, a1, 4",             // src += 4
    "addi a2, a2, 4",             // dest += 4
    "bnez a0, 2b",  // Iterate again if remaining != 0

    // fn zero_bss:
    "3:",
    "lw a0, 24(a5)",               // remaining = rt_header.bss_size
    "beqz a0, 5f",                 // Jump to call_rust_start if remaining is zero
    "lw a1, 28(a5)",               // dest = rt_header.bss_start

    // fn bss_loop_body()
    "4:",
    "sb zero, 0(a1)",              // *dest = zero
    "addi a0, a0, -1",             // remaining -= 1
    "addi a1, a1, 1",              // dest += 1
    "bnez a0, 4b",                 // Iterate again if remaining != 0

    // fn call_rust_start:
    "5:",
    // Note: rust_start must be a diverging function (i.e. return `!`)
    "jal {trampoline}",
        trampoline = sym rust_start,
    )
}

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
            #[allow(unreachable_code)] // so that fn main() -> ! does not produce a warning.
            $crate::startup::handle_main_return($name())
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

/// This is public for the sake of making `set_main!` usable in other crates.
/// It doesn't have another function.
pub fn handle_main_return<T: Termination>(result: T) -> ! {
    Termination::complete::<TockSyscalls>(result)
}

// The runtime header, which is generated by the linker script and placed at the
// beginning of the app binary.
#[repr(C)]
struct RtHeader {
    start: usize,
    initial_break: *mut (),
    stack_top: *mut (),
    data_size: usize,
    data_flash_start: *const u8,
    data_ram_start: *mut u8,
    bss_size: usize,
    bss_start: *mut u8,
}

// rust_start is the first Rust code to execute in the process. It is called
// from start, which is written directly in assembly.
#[no_mangle]
extern "C" fn rust_start() -> ! {
    extern "Rust" {
        fn libtock_unsafe_main() -> !;
        static rt_header: RtHeader;
    }

    #[cfg(not(feature = "no_debug_memop"))]
    // Safety: rt_header is defined in the linker script, valid for its type,
    // and not modified anywhere
    unsafe {
        let _ = TockSyscalls::memop_debug_stack_start(rt_header.stack_top as *const u8);
        let _ = TockSyscalls::memop_debug_heap_start(rt_header.initial_break as *const u8);
    }

    // Safety: libtock_unsafe_main is defined by the set_main! macro, and its
    // signature matches the signature in the `extern` block in this function.
    unsafe {
        libtock_unsafe_main();
    }
}

/// Function which an allocator can call to learn the initial
/// start of the heap region
pub fn get_heap_start() -> *mut () {
    extern "Rust" {
        static rt_header: RtHeader;
    }
    // Safety: rt_header is defined in the linker script, valid for its type,
    // and not modified anywhere
    unsafe { rt_header.initial_break }
}
