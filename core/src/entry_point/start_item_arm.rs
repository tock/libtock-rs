/// Tock programs' entry point. Called by the kernel at program start. Sets up
/// the stack then calls rust_start() for the remainder of setup.
#[doc(hidden)]
#[no_mangle]
#[naked]
#[link_section = ".start"]
pub unsafe extern "C" fn start() -> ! {
    asm!(
        "
	/* First, verify the process binary was loaded at the correct address. The
	 * check is performed by comparing the program counter at the start to the
	 * address of `start`, which is stored in rt_header. */
	mov r4, pc        /* r4 = address of .start + 4 (Thumb bit unset) */
	mov r5, r0        /* Save rt_header; we use r0 for syscalls */
	ldr r0, [r5, #0]  /* r0 = rt_header.start */
	add r0, #3        /* r0 = rt_header.start + 4 - 1 (for Thumb bit) */
	cmp r0, r4
	beq .Lset_brk     /* Skip error handling if pc correct */
	/* If the beq on the previous line did not jump, then the binary is not at
	 * the correct location. Report the error via LowLevelDebug then exit. */
	mov r0, #8  /* LowLevelDebug driver number */
	mov r1, #1  /* Command: print alert code */
	mov r2, #2  /* Alert code 2 (incorrect location */
	svc 2       /* Execute `command` */
	mov r0, #0  /* Operation: exit-terminate */
	svc 6       /* Execute `exit` */

.Lset_brk:
	/* memop(): set brk to rt_header's initial break value */
	mov r0, #0        /* operation: set break */
	ldr r1, [r5, #4]  /* rt_header`s initial process break */
	svc 5             /* call `memop` */

	/* Set the stack pointer */
	ldr r0, [r5, #8]  /* r0 = rt_header._stack_top */
	mov sp, r0

	/* Copy .data into place */
	ldr r0, [r5, #12]          /* remaining = rt_header.data_size */
	cbz r0, .Lzero_bss         /* Jump to zero_bss if remaining == 0 */
	ldr r1, [r5, #16]          /* src = rt_header.data_flash_start */
	ldr r2, [r5, #20]          /* dest = rt_header.data_ram_start */
.Ldata_loop_body:
	ldr r3, [r1]               /* r3 = *src */
	str r3, [r2]               /* *(dest) = r3 */
	sub r0, #4                 /* remaining -= 4 */
	add r1, #4                 /* src += 4 */
	add r2, #4                 /* dest += 4 */
	cmp r0, #0
	bne .Ldata_loop_body       /* Iterate again if remaining != 0 */

.Lzero_bss:
	ldr r0, [r5, #24]          /* remaining = rt_header.bss_size */
	cbz r0, .Lcall_rust_start  /* Jump to call_rust_start if remaining == 0 */
	ldr r1, [r5, #28]          /* dest = rt_header.bss_start */
	mov r2, #0                 /* r2 = 0 */
.Lbss_loop_body:
	strb r2, [r1]              /* *(dest) = r2 = 0 */
	sub r0, #1                 /* remaining -= 1 */
	add r1, #1                 /* dest += 1 */
	cmp r0, #0
	bne .Lbss_loop_body        /* Iterate again if remaining != 0 */

.Lcall_rust_start:
	bl rust_start
        ",
        // No clobbers because we don't return.
        options(noreturn),
    )
}
