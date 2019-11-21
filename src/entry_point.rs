use crate::memop;
use crate::syscalls;
use core::intrinsics;
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

/// Tock programs' entry point. Called by the kernel at program start. Sets up
/// the stack then calls rust_start() for the remainder of setup.
#[cfg(target_arch = "arm")]
#[doc(hidden)]
#[no_mangle]
#[naked]
#[link_section = ".start"]
pub unsafe extern "C" fn _start(
    app_start: usize,
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
        movw r0, #8       // LowLevelDebug driver number
        movw r1, #1       // LowLevelDebug 'print status code' command
        movw r2, #2       // LowLevelDebug relocation failed status code
        svc 2             // command() syscall
        .Lyield_loop:
        svc 0             // yield() syscall (in infinite loop)
        b .Lyield_loop

        .Lstack_init:
        // Compute the stacktop (stack_start). The stacktop is computed as
        // stack_size + mem_start plus padding to align the stack to a multiple
        // of 8 bytes. The 8 byte alignment is to follow ARM AAPCS:
        // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.faqs/ka4127.html
        ldr r4, [r0, #36]  // r4 = app_start->stack_size
        add r4, r4, r1     // r4 = app_start->stack_size + mem_start
        add r4, #7         // r4 = app_start->stack_size + mem_start + 7
        bic r4, r4, #7     // r4 = (app_start->stack_size + mem_start + 7) & ~0x7
        mov sp, r4         // sp = r4

        // We need to pass app_start, stacktop and app_heap_break to rust_start.
        // Temporarily store them in r6, r7 and r8
        mov r6, r0
        mov r7, sp

        // Debug support, tell the kernel the stack location
        //
        // memop(10, stacktop)
        // r7 contains stacktop
        mov r0, #10
        mov r1, r7
        svc 4

        // Debug support, tell the kernel the heap_start location
        mov r0, r6
        ldr r4, [r0, #24] // r4 = app_start->bss_start
        ldr r5, [r0, #28] // r5 = app_start->bss_size
        add r4, r4, r5    // r4 = bss_start + bss_size
        //
        // memop(11, r4)
        mov r0, #11
        mov r1, r4
        svc 4

        // Store heap_start (and soon to be app_heap_break) in r8
        mov r8, r4

        // There is a possibility that stack + .data + .bss is greater than
        // 3072. Therefore setup the initial app_heap_break to heap_start (that
        // is zero initial heap) and let rust_start determine where the actual
        // app_heap_break should go.
        //
        // Also, because app_heap_break is where the unprivileged MPU region
        // ends, in case mem_start + stack + .data + .bss is greater than
        // initial app_heap_break (mem_start + 3072), we will get a memory fault
        // in rust_start when initializing .data and .bss. Setting
        // app_heap_break to heap_start avoids that.

        // memop(0, r8)
        mov r0, #0
        mov r1, r8
        svc 4

        // NOTE: If there is a hard-fault before this point, then
        //       process_detail_fmt in kernel/src/process.rs panics which
        //       will result in us losing the PC of the instruction
        //       generating the hard-fault. Therefore any code before
        //       this point is critical code

        // Setup parameters needed by rust_start
        // r6 (app_start), r7 (stacktop), r8 (app_heap_break)
        mov r0, r6
        mov r1, r7
        mov r2, r8

        // Call rust_start
        bl rust_start"
        :                                                              // No output operands
        : "{r0}"(app_start), "{r1}"(mem_start), "{r3}"(app_heap_break) // Input operands
        : "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r12",
          "cc", "memory"                                               // Clobbers
        : "volatile"                                                   // Options
    );
    intrinsics::unreachable();
}

/// Tock programs' entry point. Called by the kernel at program start. Sets up
/// the stack then calls rust_start() for the remainder of setup.
#[cfg(target_arch = "riscv32")]
#[doc(hidden)]
#[naked]
#[no_mangle]
#[link_section = ".start"]
// The args for this function are:
//    app_start: usize,
//    mem_start: usize,
//    memory_len: usize,
//    app_heap_break: usize,
// Due to Rust issue: https://github.com/rust-lang/rust/issues/42779 we can't have
// args to the function
pub unsafe extern "C" fn _start() -> ! {
    asm!(
    // Compute the stack top.
    //
    // struct hdr* myhdr = (struct hdr*) app_start;
    // uint32_t stacktop = (((uint32_t) mem_start + myhdr->stack_size + 7) & 0xfffffff8);
    "lw   t0, 36(a0)         // t0 = myhdr->stack_size
    addi t0, t0, 7          // t0 = myhdr->stack_size + 7
    add  t0, t0, a1         // t0 = mem_start + myhdr->stack_size + 7
    li   t1, 7              // t1 = 7
    not  t1, t1             // t1 = ~0x7
    and  t0, t0, t1         // t0 = (mem_start + myhdr->stack_size + 7) & ~0x7
    //
    // Compute the app data size and where initial app brk should go.
    // This includes the GOT, data, and BSS sections. However, we can't be sure
    // the linker puts them back-to-back, but we do assume that BSS is last
    // (i.e. myhdr->got_start < myhdr->bss_start && myhdr->data_start <
    // myhdr->bss_start). With all of that true, then the size is equivalent
    // to the end of the BSS section.
    //
    // uint32_t appdata_size = myhdr->bss_start + myhdr->bss_size;
    lw   t1, 24(a0)         // t1 = myhdr->bss_start
    lw   t2, 28(a0)         // t2 = myhdr->bss_size
    lw   t3,  4(a0)         // t3 = myhdr->got_start
    add  t1, t1, t2         // t1 = bss_start + bss_size
    //
    // Move arguments we need to keep over to callee-saved locations.
    mv   s0, a0             // s0 = void* app_start
    mv   s1, t0             // s1 = stack_top
    mv   s2, a3             // s2 = app_heap_break
    //
    // Now we may want to move the stack pointer. If the kernel set the
    // `app_heap_break` larger than we need (and we are going to call `brk()`
    // to reduce it) then our stack pointer will fit and we can move it now.
    // Otherwise after the first syscall (the memop to set the brk), the return
    // will use a stack that is outside of the process accessible memory.
    //
    add t2, t0, t1          // t2 = stacktop + appdata_size
    bgt t2, a3, skip_set_sp // Compare `app_heap_break` with new brk.
                                // If our current `app_heap_break` is larger
                                // then we need to move the stack pointer
                                // before we call the `brk` syscall.
    mv  sp, t0              // Update the stack pointer

    skip_set_sp:            // Back to regularly scheduled programming.

    // Call `brk` to set to requested memory

    // memop(0, stacktop + appdata_size);
    li  a0, 4               // a0 = 4   // memop syscall
    li  a1, 0               // a1 = 0
    mv  a2, t2              // a2 = stacktop + appdata_size
    ecall                   // memop
    //
    // Debug support, tell the kernel the stack location
    //
    // memop(10, stacktop);
    li  a0, 4               // a0 = 4   // memop syscall
    li  a1, 10              // a1 = 10
    mv  a2, s1              // a2 = stacktop
    ecall                   // memop
    //
    // Debug support, tell the kernel the heap location
    //
    // memop(11, stacktop + appdata_size);
    li  a0, 4               // a0 = 4   // memop syscall
    li  a1, 11              // a1 = 10
    mv  a2, t2              // a2 = stacktop + appdata_size
    ecall                   // memop
    //
    // Setup initial stack pointer for normal execution
    // Call into the rest of startup. This should never return.
    mv   sp, s1             // sp = stacktop
    mv   a0, s0             // first arg is app_start
    mv   s0, sp             // Set the frame pointer to sp.
    mv   a1, s1             // second arg is stacktop
    mv   a2, s2             // third arg is app_heap_break
    jal  rust_start"
    :                                                              // No output operands
    :
    : "memory", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7",
      "t0", "t1", "t2", "t3", "t4", "t5", "t6", "ra"               // Clobbers
    : "volatile"                                                   // Options
    );
    intrinsics::unreachable();
}

/// Ensure an abort symbol exists.
#[cfg(target_arch = "riscv32")]
#[link_section = ".start"]
#[export_name = "abort"]
pub extern "C" fn abort() {
    unsafe {
        asm! ("
            // Simply go back to the start as if we had just booted.
            j    _start
        "
        :
        :
        :
        : "volatile");
    }
}

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
pub unsafe extern "C" fn rust_start(app_start: usize, stacktop: usize, app_heap_break: usize) -> ! {
    extern "C" {
        // This function is created internally by `rustc`. See
        // `src/lang_items.rs` for more details.
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }

    // Copy .data into its final location in RAM (determined by the linker
    // script -- should be immediately above the stack).
    let layout_header: &LayoutHeader = core::mem::transmute(app_start);

    let data_flash_start_addr = app_start + layout_header.data_sym_start;

    intrinsics::copy_nonoverlapping(
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

    // we could have also bss_end for app_heap_start
    let app_heap_start = app_heap_break;
    let app_heap_end = app_heap_break + HEAP_SIZE;

    // Tell the kernel the new app heap break.
    memop::set_brk(app_heap_end as *const u8);

    HEAP.init(app_heap_start, HEAP_SIZE);

    main(0, ptr::null());

    loop {
        syscalls::yieldk();
    }
}

use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::ptr::NonNull;
use linked_list_allocator::Heap;

#[global_allocator]
static ALLOCATOR: TockAllocator = TockAllocator;

static mut HEAP: Heap = Heap::empty();

struct TockAllocator;

unsafe impl GlobalAlloc for TockAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HEAP.allocate_first_fit(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.deallocate(NonNull::new_unchecked(ptr), layout)
    }
}
