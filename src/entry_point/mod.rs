use crate::memop;
use crate::syscalls;
use core::ptr;

// _start and rust_start are the first two procedures executed when a Tock
// application starts. _start is invoked directly by the Tock kernel; it
// performs stack setup then calls rust_start. rust_start performs data
// relocation and sets up the heap before calling the rustc-generated main.
// rust_start and _start are tightly coupled.
//
// The memory layout is controlled by the linker script.
//
// When the kernel gives control to us, we get r0-r3 values that is as follows.
//
//     +--------------+ <- (r2) mem.len()
//     | Grant        |
//     +--------------+
//     | Unused       |
//  S  +--------------+ <- (r3) app_heap_break
//  R  | Heap         |         (hardcoded to mem_start + 3072 in
//  A  +--------------|          Processs::create which could be lesser than
//  M  | .bss         |          mem_start + stack + .data + .bss)
//     +--------------|
//     | .data        |
//     +--------------+
//     | Stack        |
//     +--------------+ <- (r1) mem_start
//
//     +--------------+
//     | .text        |
//  F  +--------------+
//  L  | .crt0_header |
//  A  +--------------+ <- (r0) app_start
//  S  | Protected    |
//  H  | Region       |
//     +--------------+
//
// We want to organize the memory as follows.
//
//     +--------------+ <- app_heap_break
//     | Heap         |
//     +--------------| <- heap_start
//     | .bss         |
//     +--------------|
//     | .data        |
//     +--------------+ <- stack_start (stacktop)
//     | Stack        |
//     | (grows down) |
//     +--------------+ <- mem_start
//
// app_heap_break and mem_start are given to us by the kernel. The stack size is
// determined using pointer app_start, and is used with mem_start to compute
// stack_start (stacktop). The placement of .data and .bss are given to us by
// the linker script; the heap is located between the end of .bss and
// app_heap_break. This requires that .bss is the last (highest-address) section
// placed by the linker script.

#[cfg_attr(target_arch = "riscv32", path = "start_item_riscv32.rs")]
#[cfg_attr(target_arch = "arm", path = "start_item_arm.rs")]
#[cfg_attr(
    not(any(target_arch = "arm", target_arch = "riscv32")),
    path = "start_item_mock.rs"
)]
mod start_item;

/// The header encoded at the beginning of .text by the linker script. It is
/// accessed by rust_start() using its app_start parameter.
#[repr(C)]
struct LayoutHeader {
    got_sym_start: usize,
    got_start: usize,
    got_size: usize,
    data_sym_start: usize,
    data_start: usize,
    data_size: usize,
    bss_start: usize,
    bss_size: usize,
    reldata_start: usize,
    stack_size: usize,
}

/// Rust setup, called by _start. Uses the extern "C" calling convention so that
/// the assembly in _start knows how to call it (the Rust ABI is not defined).
/// Sets up the data segment (including relocations) and the heap, then calls
/// into the rustc-generated main(). This cannot use mutable global variables or
/// global references to globals until it is done setting up the data segment.
#[no_mangle]
unsafe extern "C" fn rust_start(app_start: usize, stacktop: usize, _app_heap_break: usize) -> ! {
    extern "C" {
        // This function is created internally by `rustc`. See
        // `src/lang_items.rs` for more details.
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }

    // Copy .data into its final location in RAM (determined by the linker
    // script -- should be immediately above the stack).
    let layout_header: &LayoutHeader = core::mem::transmute(app_start);

    let data_flash_start_addr = app_start + layout_header.data_sym_start;

    ptr::copy_nonoverlapping(
        data_flash_start_addr as *const u8,
        stacktop as *mut u8,
        layout_header.data_size,
    );

    // Zero .bss (specified by the linker script).
    let bss_end = layout_header.bss_start + layout_header.bss_size; // 1 past the end of .bss
    for i in layout_header.bss_start..bss_end {
        core::ptr::write(i as *mut u8, 0);
    }

    // TODO: Wait for rustc to have working ROPI-RWPI relocation support, then
    // implement dynamic relocations here. At the moment, rustc does not have
    // working ROPI-RWPI support, and it is not clear what that support would
    // look like at the LLVM level. Once we know what the relocation strategy
    // looks like we can write the dynamic linker.

    // Initialize the heap. Unlike libtock-c's newlib allocator, which can use
    // `sbrk` system call to dynamically request heap memory from the kernel, we
    // need to tell `linked_list_allocator` where the heap starts and ends.
    //
    // Heap size is set using `elf2tab` with `--app-heap` option, which is
    // currently at 1024. If you change the `elf2tab` heap size, make sure to
    // make the corresponding change here.
    const HEAP_SIZE: usize = 1024;

    // Make the heap start exactly at bss_end. The suggested _app_heap_break
    // is almost always going to be too big and leads to us wasting memory.
    let app_heap_start = bss_end;
    let app_heap_end = app_heap_start + HEAP_SIZE;

    // Tell the kernel the new app heap break.
    memop::set_brk(app_heap_end as *const u8);

    #[cfg(feature = "alloc")]
    crate::alloc::HEAP.init(app_heap_start, HEAP_SIZE);

    main(0, ptr::null());

    loop {
        syscalls::raw::yieldk();
    }
}
