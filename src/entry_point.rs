use crate::syscalls;
use crate::ALLOCATOR;
use core::intrinsics;
use core::ptr;

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
        // Because ROPI-RWPI support in LLVM/rustc is incomplete, Rust
        // applications must be statically linked. An offset between the
        // location the program is linked at and its actual location in flash
        // would cause references in .data and .rodata to point to the wrong
        // data. To mitigate this, this section checks that .text (and .start)
        // are loaded at the correct location. If the application was linked and
        // loaded correctly, the location of the first instruction (read using
        // the Program Counter) will match the intended location of .start. We
        // don't have an easy way to signal an error, so for now we just yield
        // if the location is wrong.
        sub r4, pc, #4    // r4 = pc
        ldr r5, =.start   // r5 = address of .start
        cmp r4, r5
        beq .Lstack_init  // Jump to stack initialization if pc was correct
        .Lyield_loop:
        svc 0             // yield() syscall
        b .Lyield_loop

        .Lstack_init:
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
    ALLOCATOR.lock().init(bss_end, app_heap_break);
    syscalls::memop(10, stack_top);
    syscalls::memop(11, bss_end);

    main(0, ptr::null());

    loop {
        syscalls::yieldk();
    }
}
