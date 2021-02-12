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
    initial_break: usize,
    stack_top: usize,
    data_size: usize,
    data_flash_start: *const u32,
    data_ram_start: *mut u32,
    bss_size: usize,
    bss_start: *mut u8,
}

#[link_section = ".start"]
#[no_mangle]
extern fn start_prototype(
    rt_header: &RtHeader,
    _memory_start: usize,
    _memory_len: usize,
    _app_break: usize,
) -> ! {
    use crate::TockSyscalls;
    use libtock_platform::{OneArgMemop, RawSyscalls, YieldType};

    let pc: usize;
    #[cfg(target_arch = "riscv32")]
    unsafe {
        asm!("auipc {}, 0", lateout(reg) pc, options(nomem, nostack, preserves_flags));
    }
    if pc != rt_header.start {
        // Binary is in an incorrect location: report an error via
        // LowLevelDebug.
        unsafe {
            TockSyscalls::four_arg_syscall(8, 1, 2, 0, 2);
        }
        // TODO: Replace with an Exit call when exit is implemented.
        loop {
            TockSyscalls::raw_yield(YieldType::Wait);
        }
    }

    // Set the app break.
    // TODO: Replace with Syscalls::memop_brk() when that is implemented.
    TockSyscalls::one_arg_memop(OneArgMemop::Brk, rt_header.initial_break);

    // Set the stack pointer.
    #[cfg(target_arch = "riscv32")]
    unsafe {
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

    extern {
        fn rust_start() -> !;
    }

    unsafe { rust_start(); }
}
