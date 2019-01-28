extern crate linked_list_allocator;

use self::linked_list_allocator::Heap;
use crate::syscalls;
use core::alloc::Alloc;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::intrinsics;
use core::mem;
use core::ptr;
use core::ptr::NonNull;

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

// Note: At the moment, rust_start is incomplete. The rest of this comment
// describes how rust_start *should* work. It does not currently perform data
// relocation (note the TODO in rust_start's source).
//
// _start and rust_start are the first two procedures executed when a Tock
// application starts. _start is invoked directly by the Tock kernel; it
// performs stack setup then calls rust_start. rust_start performs data
// relocation and sets up the heap before calling the rustc-generated main.
// rust_start and _start are tightly coupled: the order of rust_start's
// parameters is designed to simplify _start's implementation.
//
// The memory layout is controlled by the linker script and these methods. These
// are written for the following memory layout:
//
//     +----------------+ <- app_heap_break
//     | Heap           |
//     +----------------| <- heap_bottom
//     | .data and .bss |
//     +----------------+ <- stack_top
//     | Stack          |
//     | (grows down)   |
//     +----------------+ <- mem_start
//
// app_heap_break and mem_start are given to us by the kernel. The stack size is
// determined using pointer text_start, and is used with mem_start to compute
// stack_top. The placement of .data and .bss are given to us by the linker
// script; the heap is located between the end of .bss and app_heap_break. This
// requires that .bss is the last (highest-address) section placed by the linker
// script.

/// Tock programs' entry point. Called by the kernel at program start. Sets up
/// the stack then calls rust_start() for the remainder of setup.
#[doc(hidden)]
#[no_mangle]
#[naked]
#[link_section = ".start"]
pub unsafe extern "C" fn _start(
    text_start: usize,
    mem_start: usize,
    _memory_len: usize,
    app_heap_break: usize,
) -> ! {
    asm!("
        // Initialize the stack pointer. The stack pointer is computed as
        // stack_size + mem_start plus padding to align the stack to a multiple
        // of 8 bytes. The 8 byte alignment is to follow ARM AAPCS:
        // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.faqs/ka4127.html
        ldr ip, [r0, #36]  // ip = text_start->stack_size
        add ip, ip, r1     // ip = text_start->stack_size + mem_start
        add ip, #7         // ip = text_start->stack_size + mem_start + 7
        bic r1, ip, #7     // r1 = (text_start->stack_size + mem_start + 7) & ~0x7
        mov sp, r1         // sp = r1

        // Call rust_start. text_start, stack_top, and app_heap_break are
        // already in the correct registers.
        bl rust_start"
        :                                                              // No output operands
        : "{r0}"(text_start) "{r1}"(mem_start) "{r3}"(app_heap_break)  // Input operands
        : "cc" "ip" "lr" "memory" "r0" "r1" "r2" "r3"                  // Clobbers
        :                                                              // Options
    );
    intrinsics::unreachable();
}

/// The header encoded at the beginning of .text by the linker script. It is
/// accessed by rust_start() using its text_start parameter.
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
pub unsafe extern "C" fn rust_start(
    text_start: usize,
    stack_top: usize,
    _skipped: usize,
    app_heap_break: usize,
) -> ! {
    extern "C" {
        // This function is created internally by`rustc`. See `src/lang_items.rs` for more details.
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }

    // Copy .data into its final location in RAM (determined by the linker
    // script -- should be immediately above the stack).
    let layout_header: &LayoutHeader = core::mem::transmute(text_start);
    intrinsics::copy_nonoverlapping(
        (text_start + layout_header.data_sym_start) as *const u8,
        stack_top as *mut u8,
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

    // Initialize the heap and tell the kernel where everything is. The heap is
    // placed between .bss and the end of application memory.
    TockAllocator.init(bss_end, app_heap_break);
    syscalls::memop(10, stack_top);
    syscalls::memop(11, bss_end);

    main(0, ptr::null());

    loop {
        syscalls::yieldk();
    }
}
