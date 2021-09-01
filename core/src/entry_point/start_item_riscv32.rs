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
pub unsafe extern "C" fn start() -> ! {
    asm!(
        "
	/* First, verify the process binary was loaded at the correct address. The
	 * check is performed by comparing the program counter at the start to the
	 * address of `start`, which is stored in rt_header. */
	auipc s0, 0            /* s0 = pc */
	mv a5, a0
	lw s1, 0(a5)           /* s1 = rt_header.start */
	beq s0, s1, .Lset_brk  /* Skip error handling code if pc is correct */
	/* If the beq on the previous line did not jump, then the binary is not at
	 * the correct location. Report the error via LowLevelDebug then exit. */
	li a0, 8  /* LowLevelDebug driver number */
	li a1, 1  /* Command: Print alert code */
	li a2, 2  /* Alert code 2 (incorrect location) */
	li a4, 2  /* `command` class */
	ecall
	li a0, 0  /* exit-terminate */
	/* TODO: Set a completion code, once completion codes are decided */
	li a4, 6  /* `exit` class */
	ecall

.Lset_brk:
	/* memop(): set brk to rt_header's initial break value */
	li a0, 0      /* operation: set break */
	lw a1, 4(a5)  /* rt_header's initial process break */
	li a4, 5      /* `memop` class */
	ecall

	/* Set the stack pointer */
	lw sp, 8(a5)  /* sp = rt_header._stack_top */

	/* Copy .data into place. */
	lw a0, 12(a5)              /* remaining = rt_header.data_size */
	beqz a0, .Lzero_bss        /* Jump to zero_bss if remaining is zero */
	lw a1, 16(a5)              /* src = rt_header.data_flash_start */
	lw a2, 20(a5)              /* dest = rt_header.data_ram_start */
.Ldata_loop_body:
	lw a3, 0(a1)               /* a3 = *src */
	sw a3, 0(a2)               /* *dest = a3 */
	addi a0, a0, -4            /* remaining -= 4 */
	addi a1, a1, 4             /* src += 4 */
	addi a2, a2, 4             /* dest += 4 */
	bnez a0, .Ldata_loop_body  /* Iterate again if remaining != 0 */

.Lzero_bss:
	lw a0, 24(a5)               /* remaining = rt_header.bss_size */
	beqz a0, .Lcall_rust_start  /* Jump to call_Main if remaining is zero */
	lw a1, 28(a5)               /* dest = rt_header.bss_start */
.Lbss_loop_body:
	sb zero, 0(a1)              /* *dest = zero */
	addi a0, a0, -1             /* remaining -= 1 */
	addi a1, a1, 1              /* dest += 1 */
	bnez a0, .Lbss_loop_body    /* Iterate again if remaining != 0 */

.Lcall_rust_start:
	/* Note: rust_start must be a diverging function (i.e. return `!`) */
	jal rust_start",
        // No clobbers needed for a noreturn asm! block.
        options(noreturn),
    )
}

/// Ensure an abort symbol exists.
#[cfg(target_arch = "riscv32")]
#[link_section = ".start"]
#[export_name = "abort"]
pub extern "C" fn abort() {
    unsafe {
        llvm_asm! ("
            // Simply go back to the start as if we had just booted.
            j    start
        "
        :
        :
        :
        : "volatile");
    }
}
