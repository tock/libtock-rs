use core::hint;

/// Tock programs' entry point. Called by the kernel at program start. Sets up
/// the stack then calls rust_start() for the remainder of setup.
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
    llvm_asm!(
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
    bgt t1, a3, skip_set_sp // Compare `app_heap_break` with new brk.
                                // If our current `app_heap_break` is larger
                                // then we need to move the stack pointer
                                // before we call the `brk` syscall.
    mv  sp, t0              // Update the stack pointer

    skip_set_sp:            // Back to regularly scheduled programming.

    // Call `brk` to set to requested memory

    // memop(0, stacktop + appdata_size);
    li  a0, 4               // a0 = 4   // memop syscall
    li  a1, 0               // a1 = 0
    mv  a2, t1              // a2 = appdata_size
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
    li  a1, 11              // a1 = 11
    mv  a2, t1              // a2 = appdata_size
    ecall                   // memop
    //
    // Setup initial stack pointer for normal execution
    // Call into the rest of startup. This should never return.
    mv   sp, s1             // sp = stacktop
    mv   a0, s0             // first arg is app_start
    mv   s0, sp             // Set the frame pointer to sp.
    mv   a1, s1             // second arg is stacktop
    mv   a2, t1             // third arg is app_heap_break that we told the kernel
    jal  rust_start"
    :                                                              // No output operands
    :
    : "memory", "x10", "x11", "x12", "x13", "x14", "x15", "x16", "x17",
      "x5", "x6", "x7", "x28", "x29", "x30", "x31", "x1"           // Clobbers
    : "volatile"                                                   // Options
    );
    hint::unreachable_unchecked();
}

/// Ensure an abort symbol exists.
#[cfg(target_arch = "riscv32")]
#[link_section = ".start"]
#[export_name = "abort"]
pub extern "C" fn abort() {
    unsafe {
        llvm_asm! ("
            // Simply go back to the start as if we had just booted.
            j    _start
        "
        :
        :
        :
        : "volatile");
    }
}
