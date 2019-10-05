/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

MEMORY {
  FLASH (rx) : ORIGIN = 0x20430000, LENGTH = 32M
  SRAM (rwx) : ORIGIN = 0x80000000, LENGTH = 512K
}

/*
 * Any change to STACK_SIZE should be accompanied by a corresponding change to
 * `elf2tab`'s `--stack` option
 */
STACK_SIZE = 2048;

MPU_MIN_ALIGN = 1K;

INCLUDE layout.ld
