use core::hint;

/// Tock programs' entry point. Called by the kernel at program start. Sets up
/// the stack then calls rust_start() for the remainder of setup.
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
        : "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r8", "r12",
          "cc", "memory"                                               // Clobbers
        : "volatile"                                                   // Options
    );
    hint::unreachable_unchecked()
}
