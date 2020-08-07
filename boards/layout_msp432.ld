MEMORY {
  /* The application region is 64 bytes (0x40) */
  FLASH (rx) : ORIGIN = 0x00020040, LENGTH = 0x0001FFC0
  SRAM (rwx) : ORIGIN = 0x20004000, LENGTH = 0x2000
}

/*
 * Any change to STACK_SIZE should be accompanied by a corresponding change to
 * `elf2tab`'s `--stack` option
 */
STACK_SIZE = 2048;

MPU_MIN_ALIGN = 8K;

INCLUDE layout_generic.ld
