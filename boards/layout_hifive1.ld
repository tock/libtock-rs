/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

MEMORY {
  /*
   * The TBF header can change in size so use 0x40 combined with
   * --protected-region-size with elf2tab to cover a header upto that
   * size.
   *
   * Note that the SRAM address may need to be changed depending on
   * the kernel binary, check for the actual address of APP_MEMORY!
   */
  FLASH (rx) : ORIGIN = 0x20040040, LENGTH = 32M
  SRAM (rwx) : ORIGIN = 0x80002400, LENGTH = 0x1C00
}

/*
 * Any change to STACK_SIZE should be accompanied by a corresponding change to
 * `elf2tab`'s `--stack` option
 */
STACK_SIZE = 2048;

MPU_MIN_ALIGN = 1K;

INCLUDE layout_generic.ld
