#![feature(asm, alloc, allocator_api, compiler_builtins_lib, const_fn, global_allocator,
           lang_items, naked_functions)]
#![no_std]

pub mod button;
pub mod console;
pub mod electronics;
pub mod fmt;
pub mod ipc;
pub mod led;
pub mod sensors;
pub mod syscalls;
pub mod timer;
pub mod gpio;
pub mod simple_ble;

extern crate alloc;
extern crate compiler_builtins;
extern crate linked_list_allocator;

mod lang_items;

use alloc::allocator::Alloc;
use alloc::allocator::AllocErr;
use alloc::allocator::Layout;
use alloc::string::String;
use console::Console;
use core::mem::{align_of, size_of};
use core::ptr;
use linked_list_allocator::{align_up, Heap};

// None-threaded heap wrapper based on `r9` register instead of global variable
struct BaseHeap;

impl BaseHeap {
    pub unsafe fn heap(&self) -> &mut Heap {
        let heap: *mut Heap;
        asm!("mov $0, r9" : "=r"(heap) : : : "volatile");
        &mut *heap
    }

    /// Initializes an empty heap
    ///
    /// Returns the end of the heap
    ///
    /// # Unsafety
    ///
    /// This function must be called at most once and must only be used on an
    /// empty heap.
    #[inline(never)]
    pub unsafe fn init(&mut self, heap_size: usize) -> usize {
        let heap_bottom = align_up(
            self as *mut _ as usize + size_of::<Heap>(),
            align_of::<Heap>(),
        );
        self.heap().init(heap_bottom, heap_size);
        heap_bottom + heap_size
    }
}

unsafe impl<'a> Alloc for &'a BaseHeap {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        self.heap().allocate_first_fit(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.heap().deallocate(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: BaseHeap = BaseHeap;

/// Tock programs' entry point
#[doc(hidden)]
#[no_mangle]
#[naked]
pub extern "C" fn _start(mem_start: usize, app_heap_break: usize, kernel_memory_break: usize) -> ! {
    extern "C" {
        // This function is created internally by`rustc`. See `src/lang_items.rs` for more details.
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }

    unsafe {
        // Setup heap
        asm!("mov r9, $0" : : "r"(app_heap_break) : : "volatile"); // Removing this line will result in a crash that requires reflashing the ROM...

        BaseHeap.init(1024);

        let mut console = Console::new();
        console.write(String::from("\nProcess started\n===============\nmem_start           = "));
        console.write(fmt::u32_as_hex(mem_start as u32));
        console.write(String::from("\napp_heap_beak       = "));
        console.write(fmt::u32_as_hex(app_heap_break as u32));
        console.write(String::from("\nkernel_memory_break = "));
        console.write(fmt::u32_as_hex(kernel_memory_break as u32));
        console.write(String::from("\n\n"));

        main(0, ptr::null());
    }

    loop {
        syscalls::yieldk();
    }
}
