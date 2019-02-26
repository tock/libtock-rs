/* Layout for the nRF52-DK, used by the examples in this repository. */

MEMORY {
  /* The application region is 64 bytes (0x40) */
  FLASH (rx) : ORIGIN = 0x00020040, LENGTH = 0x0005FFC0
  SRAM (rwx) : ORIGIN = 0x20002000, LENGTH = 62K
}

/*
 * Any change to STACK_SIZE should be accompanied by a corresponding change to
 * `elf2tab`'s `--stack` option
 */
STACK_SIZE = 2048;

MPU_MIN_ALIGN = 8K;

INCLUDE layout.ld
