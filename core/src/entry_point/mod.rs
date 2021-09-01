use crate::syscalls;
use core::ptr;

// _start and rust_start are the first two procedures executed when a Tock
// application starts. _start is invoked directly by the Tock kernel; it
// performs stack setup and data relocation and then calls rust_start.
// rust_start calls the rustc-generated main.
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
mod start_item;

//Procedural macro to generate a function to read APP_HEAP_SIZE
libtock_codegen::make_read_env_var!("APP_HEAP_SIZE");

// rust_start is the first Rust code to execute in the process. It is called
// from start, which is written directly in assembly.
#[no_mangle]
extern "C" fn rust_start() -> ! {
    // TODO: Call memop() to inform the kernel of the stack size +
    // location. Only needed for debugging.
    extern "C" {
        // This function is created internally by `rustc`. See
        // `src/lang_items.rs` for more details.
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }
    let app_heap_size: usize = read_APP_HEAP_SIZE();
    unsafe {
        let _app_heap_start = super::memop::get_brk();
        // Tell the kernel the new app heap break.
        super::memop::increment_brk(app_heap_size);

        #[cfg(feature = "alloc_init")]
        crate::libtock_alloc_init(_app_heap_start as usize, app_heap_size);

        main(0, ptr::null());
    }
    loop {
        syscalls::raw::yield_wait();
    }
}
