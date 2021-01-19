/* Layout for the Nucleo F446re, used by the examples in this repository. */

MEMORY {
  /* The application region is 64 bytes (0x40) */
  FLASH (rx) : ORIGIN = 0x08040040, LENGTH = 255K
  SRAM (rwx) : ORIGIN = 0x20004000, LENGTH = 176K
}

MPU_MIN_ALIGN = 8K;

INCLUDE layout_generic.ld
