// This file is not compiled or tested! It is kept in this repository in case
// future libtock_runtime developers want to use it. To use this file, copy it
// into libtock_runtime's src/ directory and add mod start_prototype; to
// libtock_runtime's lib.rs.

// The `start` symbol must be written purely in assembly, because it has an ABI
// that the Rust compiler doesn't know (e.g. it does not expect the stack to be
// set up). One way to write a correct `start` implementation is to write it in
// Rust using the C ABI, compile that implementation, then tweak the assembly by
// hand. This is a Rust version of `start` for developers who are working on
// `start`.

#[repr(C)]
struct RtHeader {
    start: usize,
    initial_break: *mut (),
    stack_top: usize,
    data_size: usize,
    data_flash_start: *const u32,
    data_ram_start: *mut u32,
    bss_size: usize,
    bss_start: *mut u8,
}

#[link_section = ".start"]
#[no_mangle]
extern "C" fn start_prototype(
    rt_header: &RtHeader,
    _memory_start: usize,
    _memory_len: usize,
    _app_break: usize,
) -> ! {
    use crate::TockSyscalls;
    use libtock_platform::{RawSyscalls, syscall_class};

    let pc: usize;
    unsafe {
        #[cfg(target_arch = "arm")]
        asm!("mov {}, pc", lateout(reg) pc, options(nomem, nostack, preserves_flags));
        #[cfg(target_arch = "riscv32")]
        asm!("auipc {}, 0", lateout(reg) pc, options(nomem, nostack, preserves_flags));
    }
    if pc != rt_header.start {
        // Binary is in an incorrect location: report an error via
        // LowLevelDebug then exit.
        unsafe {
            TockSyscalls::syscall4::<syscall_class::COMMAND>([
                8 as *mut (),
                1 as *mut (),
                2 as *mut (),
                0 as *mut (),
            ]);
            TockSyscalls::syscall2::<syscall_class::EXIT>([0 as *mut (), 0 as *mut ()]);
        }
    }

    // Set the app break.
    // TODO: Replace with Syscalls::memop_brk() when that is implemented.
    unsafe {
        TockSyscalls::syscall2::<syscall_class::MEMOP>([0 as *mut (), rt_header.initial_break]);
    }

    // Set the stack pointer.
    unsafe {
        #[cfg(target_arch = "arm")]
        asm!("mov sp, {}", in(reg) rt_header.stack_top, options(nomem, preserves_flags));
        #[cfg(target_arch = "riscv32")]
        asm!("mv sp, {}", in(reg) rt_header.stack_top, options(nomem, preserves_flags));
    }

    // Copy .data into place. Uses a manual loop rather than
    // `core::ptr::copy*()` to avoid relying on `memcopy` or `memmove`.
    let mut remaining = rt_header.data_size;
    let mut src = rt_header.data_flash_start;
    let mut dest = rt_header.data_ram_start;
    while remaining > 0 {
        unsafe {
            core::ptr::write(dest, *(src));
            src = src.add(1);
            dest = dest.add(1);
        }
        remaining -= 4;
    }

    // Zero .bss. Uses a manual loop and volatile write to avoid relying on
    // `memset`.
    let mut remaining = rt_header.bss_size;
    let mut dest = rt_header.bss_start;
    while remaining > 0 {
        unsafe {
            core::ptr::write_volatile(dest, 0);
            dest = dest.add(1);
        }
        remaining -= 1;
    }

    extern "C" {
        fn rust_start() -> !;
    }

    unsafe {
        rust_start();
    }
}
