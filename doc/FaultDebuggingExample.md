Fault Debugging Example
=======================

This document shows the debugging process I (@jrvanwhy) used to find the cause
of an illegal instruction error. I wrote this document because the debugging
process demonstrates the use of valuable debugging tools (such as `objdump`) as
well as the thought process I use to debug low-level issues.

## The failure

When I attempt to run the `low_level_debug` example, I get the following app
fault (note: the early debug messages are from a work-in-progress PR, but should
be self-explanatory):

```
Package name: "low_level_debug"
TAB path: target/riscv32imac-unknown-none-elf/release/examples/low_level_debug.tab
Protected region size: 72
Found .stack section, size: 256
ELF file: "target/riscv32imac-unknown-none-elf/release/examples/low_level_debug"
TBF path: target/riscv32imac-unknown-none-elf/release/examples/low_level_debug.tbf
elf2tab command: "elf2tab" "-n" "low_level_debug" "-o" "target/riscv32imac-unknown-none-elf/release/examples/low_level_debug.tab" "--protected-region-size" "72" "--stack" "256" "target/riscv32imac-unknown-none-elf/release/examples/low_level_debug" "-v"
Spawning elf2tab
Creating "target/riscv32imac-unknown-none-elf/release/examples/low_level_debug.tbf"
Min RAM size from segments in ELF: 0 bytes
Number of writeable flash regions: 0
Entry point is in .start section
  Adding .start section. Offset: 72 (0x48). Length: 118 (0x76) bytes.
  Adding .text section. Offset: 190 (0xbe). Length: 64 (0x40) bytes.
Warning! Placing section .text at 0xbe, which is not 4-byte aligned.
Searching for .rel.X sections to add.
TBF Header:
               version:        2        0x2
           header_size:       64       0x40
            total_size:      512      0x200
                 flags:        1        0x1

        init_fn_offset:       40       0x28
        protected_size:        8        0x8
      minimum_ram_size:     2304      0x900

     start_process_ram: 2147493888 0x80002800
   start_process_flash: 537133128 0x20040048

elf2tab finished. exit status: 0
QEMU command: "tock2/tools/qemu/build/qemu-system-riscv32" "-M" "sifive_e,revb=true" "-kernel" "tock2/target/riscv32imac-unknown-none-elf/release/hifive1" "-nographic" "-device" "loader,file=target/riscv32imac-unknown-none-elf/release/examples/low_level_debug.tbf,addr=0x20040000"
Spawning QEMU
HiFive1 initialization complete.
Entering main loop.

panicked at 'Process low_level_debug had a fault', kernel/src/process.rs:1037:17
        Kernel version release-1.6-954-g17e698e8f

---| No debug queue found. You can set it with the DebugQueue component.

---| RISC-V Machine State |---
Last cause (mcause): Illegal instruction (interrupt=0, exception code=0x00000002)
Last value (mtval):  0x00000000

System register dump:
 mepc:    0x20012948    mstatus:     0x00000088
 mcycle:  0xFF33A41C    minstret:    0xFF33CBEA
 mtvec:   0x20010100
 mstatus: 0x00000088
  uie:    false  upie:   false
  sie:    false  spie:   false
  mie:    true   mpie:   true
  spp:    false
 mie:   0x00000888   mip:   0x00000000
  usoft:  false               false 
  ssoft:  false               false 
  msoft:  true                false 
  utimer: false               false 
  stimer: false               false 
  mtimer: true                false 
  uext:   false               false 
  sext:   false               false 
  mext:   true                false 

---| App Status |---
App: low_level_debug   -   [Faulted]
 Events Queued: 0   Syscall Count: 1   Dropped Callback Count: 0
 Restart Count: 0
 Last Syscall: Some(Memop { operand: 0, arg0: 2147494144 })


 ╔═══════════╤══════════════════════════════════════════╗
 ║  Address  │ Region Name    Used | Allocated (bytes)  ║
 ╚0x800037C0═╪══════════════════════════════════════════╝
             │ ▼ Grant         960 |    960          
  0x80003400 ┼───────────────────────────────────────────
             │ Unused
  0x80002900 ┼───────────────────────────────────────────
             │ ▲ Heap            ? |      ?               S
  ?????????? ┼─────────────────────────────────────────── R
             │ Data              ? |      ?               A
  ?????????? ┼─────────────────────────────────────────── M
             │ ▼ Stack           ? |      ?
  0x80002900 ┼───────────────────────────────────────────
             │ Unused
  0x80002800 ┴───────────────────────────────────────────
             .....
  0x20040200 ┬─────────────────────────────────────────── F
             │ App Flash       440                        L
  0x20040048 ┼─────────────────────────────────────────── A
             │ Protected        72                        S
  0x20040000 ┴─────────────────────────────────────────── H

 R0 : 0x00000000    R16: 0x00000000
 R1 : 0x200400BE    R17: 0x00000000
 R2 : 0x80002900    R18: 0x00000000
 R3 : 0x00000000    R19: 0x00000000
 R4 : 0x00000000    R20: 0x00000000
 R5 : 0x00000000    R21: 0x00000000
 R6 : 0x00000000    R22: 0x00000000
 R7 : 0x00000000    R23: 0x00000000
 R8 : 0x20040068    R24: 0x00000000
 R9 : 0x20040068    R25: 0x00000000
 R10: 0x00000000    R26: 0x00000000
 R11: 0x80002900    R27: 0x00000000
 R12: 0x00000FC0    R28: 0x00000000
 R13: 0x80003400    R29: 0x00000000
 R14: 0x00000005    R30: 0x00000000
 R15: 0x20040048    R31: 0x00000000
 PC : 0x200400F6    SP : 0x80002900

 mcause: 0x00000002 (Illegal instruction)
 mtval:  0x00000000

 PMP regions:
  [0]: addr=0x20040000, size=0x00000200, cfg=0xD (r-x)
  [1]: addr=0x80002800, size=0x00000100, cfg=0xB (rw-)
  <unset>
  <unset>

To debug, run `make lst` in the app's folder
and open the arch.0x20040048.0x80002800.lst file.
```

## Debugging process

When I see an illegal instruction error, my first thought is "did the code just
jump to an address that is not code?". The kernel's output indicates that `pc`
is `0x200400F6`. To see if `0x200400F6` is code, we disassemble the binary:

```
jrvanwhy@penguin:~/libtock-rs$ riscv64-unknown-elf-objdump -d -j .start -j .text target/riscv32imac-unknown-none-elf/release/examples/low_level_debug

target/riscv32imac-unknown-none-elf/release/examples/low_level_debug:     file format elf32-littleriscv


Disassembly of section .start:

20040048 <rt_header>:
20040048:       0068                    addi    a0,sp,12
2004004a:       2004                    fld     fs1,0(s0)
2004004c:       2900                    fld     fs0,16(a0)
2004004e:       8000                    0x8000
20040050:       2900                    fld     fs0,16(a0)
20040052:       8000                    0x8000
20040054:       0000                    unimp
20040056:       0000                    unimp
20040058:       2900                    fld     fs0,16(a0)
2004005a:       8000                    0x8000
2004005c:       2900                    fld     fs0,16(a0)
2004005e:       8000                    0x8000
20040060:       0000                    unimp
20040062:       0000                    unimp
20040064:       2900                    fld     fs0,16(a0)
20040066:       8000                    0x8000

20040068 <start>:
20040068:       00000417                auipc   s0,0x0
2004006c:       87aa                    mv      a5,a0
2004006e:       4384                    lw      s1,0(a5)
20040070:       00940c63                beq     s0,s1,20040088 <start+0x20>
20040074:       4521                    li      a0,8
20040076:       4585                    li      a1,1
20040078:       4609                    li      a2,2
2004007a:       4709                    li      a4,2
2004007c:       00000073                ecall
20040080:       4501                    li      a0,0
20040082:       4719                    li      a4,6
20040084:       00000073                ecall
20040088:       4501                    li      a0,0
2004008a:       43cc                    lw      a1,4(a5)
2004008c:       4715                    li      a4,5
2004008e:       00000073                ecall
20040092:       0087a103                lw      sp,8(a5)
20040096:       47c8                    lw      a0,12(a5)
20040098:       c909                    beqz    a0,200400aa <start+0x42>
2004009a:       4b8c                    lw      a1,16(a5)
2004009c:       4bd0                    lw      a2,20(a5)
2004009e:       4194                    lw      a3,0(a1)
200400a0:       c214                    sw      a3,0(a2)
200400a2:       1571                    addi    a0,a0,-4
200400a4:       0591                    addi    a1,a1,4
200400a6:       0611                    addi    a2,a2,4
200400a8:       f97d                    bnez    a0,2004009e <start+0x36>
200400aa:       4f88                    lw      a0,24(a5)
200400ac:       c519                    beqz    a0,200400ba <start+0x52>
200400ae:       4fcc                    lw      a1,28(a5)
200400b0:       00058023                sb      zero,0(a1)
200400b4:       157d                    addi    a0,a0,-1
200400b6:       0585                    addi    a1,a1,1
200400b8:       fd65                    bnez    a0,200400b0 <start+0x48>
200400ba:       03c000ef                jal     ra,200400f6 <rust_start>

Disassembly of section .text:

200400c0 <_ZN15low_level_debug4main17h52d4e61b1cb7ceefE>:
200400c0:       4709                    li      a4,2
200400c2:       4521                    li      a0,8
200400c4:       4605                    li      a2,1
200400c6:       4589                    li      a1,2
200400c8:       4681                    li      a3,0
200400ca:       00000073                ecall
200400ce:       4709                    li      a4,2
200400d0:       468d                    li      a3,3
200400d2:       4521                    li      a0,8
200400d4:       4609                    li      a2,2
200400d6:       458d                    li      a1,3
200400d8:       00000073                ecall
200400dc:       8082                    ret

200400de <libtock_unsafe_main>:
200400de:       1141                    addi    sp,sp,-16
200400e0:       c606                    sw      ra,12(sp)
200400e2:       00000097                auipc   ra,0x0
200400e6:       fde080e7                jalr    -34(ra) # 200400c0 <_ZN15low_level_debug4main17h52d4e61b1cb7ceefE>
200400ea:       4719                    li      a4,6
200400ec:       4501                    li      a0,0
200400ee:       4581                    li      a1,0
200400f0:       00000073                ecall
        ...

200400f6 <rust_start>:
200400f6:       00000097                auipc   ra,0x0
200400fa:       fe8080e7                jalr    -24(ra) # 200400de <libtock_unsafe_main>
        ...
```

In the ELF file, `0x200400f6` is the beginning of a valid instruction. So maybe
something is going wrong in the ELF to TBF conversion, or in the deployment?

## Address checking

The [RISC-V entry
point](https://github.com/tock/libtock-rs/blob/b0f8593c1c5dc2a4ded1305809841202107d7c75/runtime/asm/asm_riscv32.S)
has logic to verify the program counter is incorrect. If that logic executed,
then we know the TBF file was deployed to the correct location in flash. But how
do we know it executed, rather than the kernel trying to start execution at
`0x200400f6` immediately?

We can see some evidence of that in the kernel's output:

```
Last Syscall: Some(Memop { operand: 0, arg0: 2147494144 })
```

The only place that Memop calls occur in `low_level_debug` is in `asm_riscv32.S`:

```
.section .start, "ax"
.globl start
start:
    /* First, verify the process binary was loaded at the correct address. The
     * check is performed by comparing the program counter at the start to the
     * address of `start`, which is stored in rt_header. */
    auipc s0, 0            /* s0 = pc */
    mv a5, a0              /* Save rt_header so syscalls don't overwrite it */
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
```

Notably, the initial process break is loaded from `4(a5)`, and `a5` is set
*before* the `pc` check runs. So if the memory break is correct, then we can be
fairly confident the `pc` check ran. `2147494144` is `0x80002900`, which is 256
bytes past the beginning of the process binary's flash region (from
`runtime/layouts/hifive1.ld`):

```
MEMORY {
  /* Note that the SRAM address may need to be changed depending on
   * the kernel binary, check for the actual address of APP_MEMORY!
   */
  FLASH (X) : ORIGIN = 0x20040000, LENGTH = 32M
  RAM   (W) : ORIGIN = 0x80002800, LENGTH = 0x1800
}
```

That seems completely reasonable. Next, I suspect an issue in the ELF -> TBF
conversion.

## Decoding the TBF file

This is the point where the tiny size of `low_level_debug` becomes hugely
beneficial. We hexdump the entirety of the TBF file:

```
jrvanwhy@penguin:~/libtock-rs$ hexdump -C target/riscv32imac-unknown-none-elf/release/examples/low_level_debug.tbf | tee ~/decoded_tbf
00000000  02 00 40 00 00 02 00 00  01 00 00 00 62 03 4d ff  |..@.........b.M.|
00000010  01 00 0c 00 28 00 00 00  08 00 00 00 00 09 00 00  |....(...........|
00000020  03 00 0f 00 6c 6f 77 5f  6c 65 76 65 6c 5f 64 65  |....low_level_de|
00000030  62 75 67 00 05 00 08 00  00 28 00 80 48 00 04 20  |bug......(..H.. |
00000040  00 00 00 00 00 00 00 00  68 00 04 20 00 29 00 80  |........h.. .)..|
00000050  00 29 00 80 00 00 00 00  00 29 00 80 00 29 00 80  |.).......)...)..|
00000060  00 00 00 00 00 29 00 80  17 04 00 00 aa 87 84 43  |.....).........C|
00000070  63 0c 94 00 21 45 85 45  09 46 09 47 73 00 00 00  |c...!E.E.F.Gs...|
00000080  01 45 19 47 73 00 00 00  01 45 cc 43 15 47 73 00  |.E.Gs....E.C.Gs.|
00000090  00 00 03 a1 87 00 c8 47  09 c9 8c 4b d0 4b 94 41  |.......G...K.K.A|
000000a0  14 c2 71 15 91 05 11 06  7d f9 88 4f 19 c5 cc 4f  |..q.....}..O...O|
000000b0  23 80 05 00 7d 15 85 05  65 fd ef 00 c0 03 09 47  |#...}...e......G|
000000c0  21 45 05 46 89 45 81 46  73 00 00 00 09 47 8d 46  |!E.F.E.Fs....G.F|
000000d0  21 45 09 46 8d 45 73 00  00 00 82 80 41 11 06 c6  |!E.F.Es.....A...|
000000e0  97 00 00 00 e7 80 e0 fd  19 47 01 45 81 45 73 00  |.........G.E.Es.|
000000f0  00 00 00 00 97 00 00 00  e7 80 80 fe 00 00 00 00  |................|
00000100  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
*
00000200
```

I then use a text editor to remove the last two lines (which represent padding
space), and change the high bits of the addresses to match the addresses the TBF
file is deployed to:

```
20040000  02 00 40 00 00 02 00 00  01 00 00 00 62 03 4d ff  |..@.........b.M.|
20040010  01 00 0c 00 28 00 00 00  08 00 00 00 00 09 00 00  |....(...........|
20040020  03 00 0f 00 6c 6f 77 5f  6c 65 76 65 6c 5f 64 65  |....low_level_de|
20040030  62 75 67 00 05 00 08 00  00 28 00 80 48 00 04 20  |bug......(..H.. |
20040040  00 00 00 00 00 00 00 00  68 00 04 20 00 29 00 80  |........h.. .)..|
20040050  00 29 00 80 00 00 00 00  00 29 00 80 00 29 00 80  |.).......)...)..|
20040060  00 00 00 00 00 29 00 80  17 04 00 00 aa 87 84 43  |.....).........C|
20040070  63 0c 94 00 21 45 85 45  09 46 09 47 73 00 00 00  |c...!E.E.F.Gs...|
20040080  01 45 19 47 73 00 00 00  01 45 cc 43 15 47 73 00  |.E.Gs....E.C.Gs.|
20040090  00 00 03 a1 87 00 c8 47  09 c9 8c 4b d0 4b 94 41  |.......G...K.K.A|
200400a0  14 c2 71 15 91 05 11 06  7d f9 88 4f 19 c5 cc 4f  |..q.....}..O...O|
200400b0  23 80 05 00 7d 15 85 05  65 fd ef 00 c0 03 09 47  |#...}...e......G|
200400c0  21 45 05 46 89 45 81 46  73 00 00 00 09 47 8d 46  |!E.F.E.Fs....G.F|
200400d0  21 45 09 46 8d 45 73 00  00 00 82 80 41 11 06 c6  |!E.F.Es.....A...|
200400e0  97 00 00 00 e7 80 e0 fd  19 47 01 45 81 45 73 00  |.........G.E.Es.|
200400f0  00 00 00 00 97 00 00 00  e7 80 80 fe 00 00 00 00  |................|
20040100  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
```

At this point, I start going through the TBF file byte-by-byte, using the [TBF
documentation](https://github.com/tock/tock/blob/master/doc/TockBinaryFormat.md)
to decode it and add notes describing what each byte means. Once I make it past
the headers, I also compare the bytes against the disassembly, looking for a
difference. Eventually, I find one:

```
TBF headers:
20040000  02 00 40 00 00 02 00 00                          Version 2, header len 64, TBF len 512
20040008  01 00 00 00 62 03 4d ff                          Process enabled, checksum
20040010  01 00 0c 00 28 00 00 00                          Main element, Data len 12, _start offset 0x28
20040018  08 00 00 00 00 09 00 00                          Protected size 8, Minimum RAM size 2304
20040020  03 00 0f 00                                      Package name, len 15
20040024  6c 6f 77 5f 6c 65 76 65 6c 5f 64 65 62 75 67 00  "low_level_debug", padding
20040034  05 00 08 00 00 28 00 80 48 00 04 20              Fixed addrs, Len 8, RAM 0x80002800, Flash 0x20040048
20040040  00 00 00 00 00 00 00 00                          Post-header padding

rt_header:
20040048  68 00 04 20 00 29 00 80 00 29 00 80 00 00 00 00  rt_header (first 16 bytes)
20040058  00 29 00 80 00 29 00 80 00 00 00 00 00 29 00 80  rt_header (second 16 bytes)

start:
20040068  17 04 00 00 aa 87 84 43
20040070  63 0c 94 00 21 45 85 45  09 46 09 47 73 00 00 00
20040080  01 45 19 47 73 00 00 00  01 45 cc 43 15 47 73 00
20040090  00 00 03 a1 87 00 c8 47  09 c9 8c 4b d0 4b 94 41
200400a0  14 c2 71 15 91 05 11 06  7d f9 88 4f 19 c5 cc 4f
200400b0  23 80 05 00 7d 15 85 05  65 fd ef 00 c0 03

.text (misplaced!):
200400be  09 47
200400c0  21 45 05 46 89 45 81 46  73 00 00 00 09 47 8d 46
200400d0  21 45 09 46 8d 45 73 00  00 00 82 80 41 11 06 c6
200400e0  97 00 00 00 e7 80 e0 fd  19 47 01 45 81 45 73 00
200400f0  00 00 00 00 97 00 00 00  e7 80 80 fe 00 00 00 00
20040100  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
20040110  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
20040120  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
20040130  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
20040140  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
20040150  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
20040160  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
20040170  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
20040180  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
20040190  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
200401a0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
200401b0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
200401c0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
200401d0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
200401e0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
200401f0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
```

`.text` in the TBF starts at `0x200400be`, but in the ELF file it starts at
`0x200400c0`! There should be 2 more bytes of padding between `.start` and
`.text`, so that `.text` is located at a multiple of 4 bytes.

## The fix

It turns out this is a known issue in `elf2tab`, and was fixed in
https://github.com/tock/elf2tab/pull/35. I resolved the fault by updating
`elf2tab`.
