#![feature(asm,alloc,compiler_builtins_lib,const_fn,global_allocator,lang_items,naked_functions)]
#![no_std]

pub mod syscalls;
pub mod console;
pub mod timer;
pub mod led;

extern crate alloc;
extern crate compiler_builtins;
extern crate linked_list_allocator;

mod lang_items;

use core::ptr;
use core::mem::size_of;
use linked_list_allocator::BaseHeap;

#[global_allocator]
static ALLOCATOR : BaseHeap = BaseHeap;

/// Tock programs' entry point
#[doc(hidden)]
#[no_mangle]
#[naked]
pub extern "C" fn _start(mem_start: usize, app_heap_break: usize,
                         _kernel_memory_break: usize) -> ! {

    extern "C" {
        // NOTE `rustc` forces this signature on us. See `src/lang_items.rs`
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }

    unsafe {
        // Setup stack
        syscalls::memop(0, mem_start + 2048);

        let new_stack = mem_start + 1024;
        asm!("mov sp, $0" : : "r"(new_stack) : : "volatile");
        syscalls::memop(10, new_stack);

        asm!("mov r9, $0" : : "r"(new_stack) : : "volatile");
        syscalls::memop(11, new_stack);


        let heap_start = new_stack + size_of::<usize>();
        let heap_size = 1024;
        BaseHeap.init(heap_start, heap_size);

        // arguments are not used in Tock applications
        main(0, ptr::null());
    }

    loop {
        ::syscalls::yieldk();
    }
}
