/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

MEMORY {
  /* The TBF header region is 32 bytes (0x20) */
  FLASH (rx) : ORIGIN = 0x20430020, LENGTH = 32M
  SRAM (rwx) : ORIGIN = 0x80000000, LENGTH = 512K
}

/*
 * Any change to STACK_SIZE should be accompanied by a corresponding change to
 * `elf2tab`'s `--stack` option
 */
STACK_SIZE = 2048;

MPU_MIN_ALIGN = 1K;

INCLUDE layout.ld
