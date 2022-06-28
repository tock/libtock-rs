/* rt_header is defined by the general linker script (libtock_layout.ld). It has
 * the following layout:
 *
 *     Field                       | Offset
 *     ------------------------------------
 *     Address of the start symbol |      0
 *     Initial process break       |      4
 *     Top of the stack            |      8
 *     Size of .data               |     12
 *     Start of .data in flash     |     16
 *     Start of .data in ram       |     20
 *     Size of .bss                |     24
 *     Start of .bss in ram        |     28
 */

/* start is the entry point -- the first code executed by the kernel. The kernel
 * passes arguments through 4 registers:
 *
 *     r0  Pointer to beginning of the process binary's code. The linker script
 *         locates rt_header at this address.
 *
 *     r1  Address of the beginning of the process's usable memory region.
 *     r2  Size of the process' allocated memory region (including grant region)
 *     r3  Process break provided by the kernel.
 *
 * We currently only use the value in r0. It is copied into r5 early on because
 * r0 is needed to invoke system calls.
 *
 * To be compatible with ARMv6 Thumb-1, we the cmp and beq instructions
 * instead of cbz in two places. This increases the code size with 4 bytes,
 * but allows us to use it on Cortex-M0+ processors.
 */
.section .start, "ax"
.global start
.thumb_func
start:
	/* First, verify the process binary was loaded at the correct address. The
	 * check is performed by comparing the program counter at the start to the
	 * address of `start`, which is stored in rt_header. */
	mov r4, pc        /* r4 = address of .start + 4 (Thumb bit unset) */
	mov r5, r0        /* Save rt_header; we use r0 for syscalls */
	ldr r0, [r5, #0]  /* r0 = rt_header.start */
	adds r0, #4       /* r0 = rt_header.start + 4 */
	cmp r0, r4        /* Skip error handling if pc correct */
	beq .Lset_brk     
	/* If the beq on the previous line did not jump, then the binary is not at
	 * the correct location. Report the error via LowLevelDebug then exit. */
	movs r0, #8  /* LowLevelDebug driver number */
	movs r1, #1  /* Command: print alert code */
	movs r2, #2  /* Alert code 2 (incorrect location */
	svc 2        /* Execute `command` */
	movs r0, #0  /* Operation: exit-terminate */
	movs r1, #1  /* Completion code: FAIL */
	svc 6        /* Execute `exit` */

.Lset_brk:
	/* memop(): set brk to rt_header's initial break value */
	movs r0, #0       /* operation: set break */
	ldr r1, [r5, #4]  /* rt_header`s initial process break */
	svc 5             /* call `memop` */

	/* Set the stack pointer */
	ldr r0, [r5, #8]  /* r0 = rt_header._stack_top */
	mov sp, r0

	/* Copy .data into place */
	ldr r0, [r5, #12]          /* remaining = rt_header.data_size */
	cmp r0, #0                 /* Jump to zero_bss if remaining == 0 */	
	beq .Lzero_bss         
	ldr r1, [r5, #16]          /* src = rt_header.data_flash_start */
	ldr r2, [r5, #20]          /* dest = rt_header.data_ram_start */
.Ldata_loop_body:
	ldr r3, [r1]               /* r3 = *src */
	str r3, [r2]               /* *(dest) = r3 */
	subs r0, #4                /* remaining -= 4 */
	adds r1, #4                /* src += 4 */
	adds r2, #4                /* dest += 4 */
	cmp r0, #0
	bne .Ldata_loop_body       /* Iterate again if remaining != 0 */

.Lzero_bss:
	ldr r0, [r5, #24]          /* remaining = rt_header.bss_size */
	cmp r0, #0                 /* Jump to call_rust_start if remaining == 0 */
	beq .Lcall_rust_start  
	ldr r1, [r5, #28]          /* dest = rt_header.bss_start */
	movs r2, #0                /* r2 = 0 */
.Lbss_loop_body:
	strb r2, [r1]              /* *(dest) = r2 = 0 */
	subs r0, #1                /* remaining -= 1 */
	adds r1, #1                /* dest += 1 */
	cmp r0, #0
	bne .Lbss_loop_body        /* Iterate again if remaining != 0 */

.Lcall_rust_start:
	bl rust_start
