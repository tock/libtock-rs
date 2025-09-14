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
 * passes the following arguments onto the stack:
 *
 *  esp+4  Pointer to beginning of the process binary's code. The linker script
 *         locates rt_header at this address.
 *
 *     +8  Address of the beginning of the process's usable memory region.
 *     +12 Size of the process' allocated memory region (including grant region)
 *     +16 Process break provided by the kernel.
 *
 * We currently only use the value in esp+4.
 */

/* int 0x03 is used to trigger a breakpoint which is promoted to a hard fault in the
   absence of a debugger. This is useful to fault at failure cases where there is no
   recovery path.
 */

/* Specify that the start code is allocated and executable (ax),
 * and that it should be placed in the .start section of the binary.
 */

.section .start, "ax"
.globl start
start:
   /*
    * Verify that the binary was loaded to the correct
    * address. We can do this by using the call command
    * and grabbing the EIP off of the stack. The eip
    * will be the "length of the call instruction" (5 bytes) 
    * ahead of the actual start of the program.
    */

    call .Lget_eip          // 1 byte for the opcode + 4 bytes for the relative offset
                            // = 5 byte long instruction
.Lget_eip:
    popl %eax               // eax = eip
    subl $5, %eax           // eax = eip - 5 byte instruction
    movl 4(%esp), %ebx      // ebx = rt_header (top of memory)
    movl 0(%ebx), %ecx      // ecx = rt_header.start
    cmpl %ecx, %eax
    je .Lset_brk
    /* If the binary is not at the correct location, report the error via LowLevelDebug
     * then exit. */
    pushl %eax             // eip, not consumed by the syscall, but is seen in trace
    pushl $2               // Code 0x02 (app was not installed in the correct location)
    pushl $1               // Minor number: Alert code 
    pushl $8               // Major number: LowLevelDebug driver
    mov $2, %eax           // Command syscall
    int $0x40
    addl $16, %esp
    pushl $0
    pushl $0
    pushl $1              // Completion code: FAIL
    pushl $0              // exit-terminate
    mov $6, %eax          // Exit syscall
    int $0x40
    addl $16, %esp
    int $0x03              // If we return, trigger a fault
    

    /* Set brk to rt_header initial break value */
.Lset_brk:
    movl 4(%ebx), %ecx      // ecx = initial process break
    pushl $0
    pushl $0
    pushl %ecx              // push initial process break
    pushl $0
    movl  $5, %eax          // memop
    int $0x40

    /* Set the stack pointer */
    mov 8(%ebx), %esp

.Lzero_bss:
    /* Zero out .bss */
    movl 24(%ebx), %ecx     // ecx = remaining = rt_header.bss_size
    cmpl $0, %ecx
    je .Lcopy_data          // If there is no .bss, jump to copying .data
    movl 28(%ebx), %edi     // edi = dst = rt_header.bss_start
    shrl $2, %ecx           // ecx = remaining / 4 = number of words to zero
    cld                     // Clear the direction flag
    xorl %eax, %eax         // eax = 0, value to set .bss to
    rep stosl               // Zero out the .bss_size
    movl 24(%ebx), %ecx     // ecx = remaining = rt_header.bss_size
    andl $3, %ecx           // ecx = remaining % 4 = number of bytes to zero
    rep stosb               // Zero out the remaining bytes

.Lcopy_data:
    /* Copy .data into place */
    movl 12(%ebx), %ecx     // ecx = rt_header.data_size
    cmpl $0, %ecx
    je .Lcall_rust_start
    movl 16(%ebx), %esi     // esi = src = rt_header.data_flash_start
    movl 20(%ebx), %edi     // edi = dst = rt_header.data_ram_start
    shrl $2, %ecx           // ecx = rt_header.data_size / 4 = number of words to copy
    cld                     // Clear the direction flag
    rep movsl               // Copy data from flash to ram
    movl 12(%ebx), %ecx     // ecx = rt_header.data_size
    andl $3, %ecx           // ecx = rt_header.data_size % 4 = number of bytes to copy
    rep movsb               // Copy the remaining bytes
    
.Lcall_rust_start:
    jmp rust_start
    int $0x03               // If we return, trigger a fault