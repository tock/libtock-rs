#![feature(asm, alloc, allocator_api, compiler_builtins_lib, const_fn, global_allocator,
           lang_items, naked_functions)]
#![no_std]

pub mod buttons;
pub mod console;
pub mod debug;
pub mod electronics;
pub mod fmt;
pub mod gpio;
pub mod ipc;
pub mod led;
pub mod result;
pub mod sensors;
pub mod simple_ble;
pub mod syscalls;
pub mod timer;
pub mod util;
pub mod ble_parser;

extern crate alloc;
extern crate compiler_builtins;
#[cfg(target_os = "tock")]
extern crate linked_list_allocator;
#[cfg(not(target_os = "tock"))]
extern crate std;

#[cfg(target_os = "tock")]
mod lang_items;

#[cfg(target_os = "tock")]
use alloc::allocator::Alloc;
#[cfg(target_os = "tock")]
use alloc::allocator::AllocErr;
#[cfg(target_os = "tock")]
use alloc::allocator::Layout;
use alloc::string::String;
use console::Console;
use core::mem;
use core::ptr;
#[cfg(target_os = "tock")]
use linked_list_allocator::Heap;

// None-threaded heap wrapper based on `r9` register instead of global variable
#[cfg(target_os = "tock")]
struct StackOwnedHeap;

#[cfg(target_os = "tock")]
impl StackOwnedHeap {
    pub unsafe fn heap(&self) -> &mut Heap {
        let mut heap: *mut Heap;
        asm!("mov $0, r9" : "=r"(heap) : : : "volatile");
        heap = ((heap as u32) - HEAP_SIZE as u32 + HEAP_OFFSET as u32) as *mut Heap;
        &mut *heap
    }

    /// Initializes an empty heap
    ///
    /// # Unsafety
    ///
    /// This function must be called at most once. The heap_buffer must remain valid until the end of the process.
    #[inline(never)]
    unsafe fn init(&mut self, heap_buffer: &mut [u8]) {
        let heap_location = heap_buffer.as_ptr() as usize;

        let heap_bottom = heap_location + mem::size_of::<Heap>();
        let heap_top = heap_location + heap_buffer.len();
        *self.heap() = Heap::new(heap_bottom, heap_top - heap_bottom);
    }
}

#[cfg(target_os = "tock")]
unsafe impl<'a> Alloc for &'a StackOwnedHeap {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        self.heap().alloc(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.heap().dealloc(ptr, layout)
    }
}

#[cfg(target_os = "tock")]
#[global_allocator]
static ALLOCATOR: StackOwnedHeap = StackOwnedHeap;

const HEAP_SIZE: usize = 0x200;
const HEAP_OFFSET: usize = 0x1c;
const STACK_SIZE: usize = 0x400;

/// Tock programs' entry point
#[cfg(target_os = "tock")]
#[doc(hidden)]
#[no_mangle]
#[naked]
#[link_section = ".start"]
pub extern "C" fn _start(
    _text_start: usize,
    mem_start: usize,
    _memory_len: usize,
    _app_heap_break: usize,
) -> ! {
    unsafe {
        asm!("mov sp, $0" : : "r"(mem_start+HEAP_SIZE+STACK_SIZE) : "memory" :  "volatile" );
        asm!("mov r9, $0" : : "r"(mem_start+HEAP_SIZE+STACK_SIZE) : : "volatile");
        run_with_new_stack()
    }
}

#[cfg(target_os = "tock")]
#[inline(never)]
unsafe fn run_with_new_stack() -> ! {
    extern "C" {
        // This function is created internally by`rustc`. See `src/lang_items.rs` for more details.
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }
    let mut heap: [u8; HEAP_SIZE as usize] = [0; HEAP_SIZE as usize];

    StackOwnedHeap.init(&mut heap);

    let mut console = Console::new();
    console.write(String::from(
        "\nProcess started\n===============\nHeap position: \n",
    ));
    console.write(fmt::u32_as_hex(heap.as_ptr() as u32));
    console.write(String::from("\n"));

    main(0, ptr::null());

    loop {
        syscalls::yieldk();
    }
}
