/* Layout for the Nucleo F429zi, used by the examples in this repository. */

MEMORY {
  /* The application region is 64 bytes (0x40) */
  FLASH (rx) : ORIGIN = 0x08040040, LENGTH = 255K
  SRAM (rwx) : ORIGIN = 0x20004000, LENGTH = 112K
}

/*
 * Any change to STACK_SIZE should be accompanied by a corresponding change to
 * `elf2tab`'s `--stack` option
 */
STACK_SIZE = 2048;

MPU_MIN_ALIGN = 8K;

INCLUDE layout_generic.ld
