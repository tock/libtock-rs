extern crate linked_list_allocator;

use self::linked_list_allocator::Heap;
use core::alloc::Alloc;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::mem;
use core::ptr;
use core::ptr::NonNull;
use debug;
use syscalls;

const HEAP_SIZE: usize = 0x400;
const SENTINEL: usize = 0x8000_0000; // Mask for addresses that need a relocation fixup
const TBF_HEADER_SIZE_MASK: usize = 0x0000_0FFF; // Assumes that TBF header size is encoded in the last bytes of the text start address

// None-threaded heap wrapper based on `r9` register instead of global variable
pub(crate) struct TockAllocator;

impl TockAllocator {
    unsafe fn heap(&self) -> &mut Heap {
        let heap: *mut Heap;
        asm!("mov $0, r9" : "=r"(heap) : : : "volatile");
        &mut *heap
    }

    /// Initializes an empty heap
    ///
    /// # Unsafety
    ///
    /// This function must be called at most once. The memory between [`heap_location`] and [`heap_top`] must not overlap with any other memory section.
    #[inline(never)]
    unsafe fn init(&mut self, heap_bottom: usize, heap_top: usize) {
        asm!("mov r9, $0" : : "r"(heap_bottom) : : "volatile");

        let effective_heap_bottom = heap_bottom + mem::size_of::<Heap>();

        let heap = heap_bottom as *mut Heap;
        *heap = Heap::new(effective_heap_bottom, heap_top - effective_heap_bottom);
    }
}

unsafe impl GlobalAlloc for TockAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.heap().alloc(layout).unwrap().as_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.heap().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

#[repr(C)]
struct Header {
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

/// Tock programs' entry point
#[doc(hidden)]
#[no_mangle]
#[naked]
#[link_section = ".start"]
pub unsafe extern "C" fn _start(
    text_start: usize,
    mem_start: usize,
    _memory_len: usize,
    _app_heap_break: usize,
) -> ! {
    let stack_size = *(text_start as *const usize).offset(9);
    let real_stack_top = mem_start + stack_size;
    // Set the effective stack top below the real stack stop and use the space in between for the heap
    let effective_stack_top = real_stack_top - HEAP_SIZE;
    TockAllocator.init(effective_stack_top, real_stack_top);

    asm!("mov sp, $0" : : "r"(effective_stack_top) : "memory" :  "volatile" );

    syscalls::memop(0, effective_stack_top + HEAP_SIZE);
    syscalls::memop(11, effective_stack_top + HEAP_SIZE);
    syscalls::memop(10, effective_stack_top);

    let header = &*(text_start as *const Header);

    // FIXME: relies on empty got, compute begin of data section correctly in linker script
    let flash_vtable_location = text_start + header.data_sym_start;

    let tbf_header_size = text_start & TBF_HEADER_SIZE_MASK;
    let data_in_memory = real_stack_top + tbf_header_size;

    debug::print_as_hex(text_start);
    debug::print_as_hex(mem_start);

    // copy and fixup data segment
    // FIXME: only modify vtable entries
    for i in 0..header.data_size / 4 {
        let ram_position = (data_in_memory as *mut usize).add(i);
        let flash_position = (flash_vtable_location as *const usize).add(i);

        let value_in_flash = ptr::read(flash_position);

        let value_needs_fixup = value_in_flash & SENTINEL != 0;
        let value_in_ram = if value_needs_fixup {
            text_start + (value_in_flash ^ SENTINEL)
        } else {
            value_in_flash
        };

        ptr::write(ram_position, value_in_ram);
    }

    start_program()
}

unsafe fn start_program() -> ! {
    extern "C" {
        // This function is created internally by`rustc`. See `src/lang_items.rs` for more details.
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }

    main(0, ptr::null());

    loop {
        syscalls::yieldk();
    }
}
