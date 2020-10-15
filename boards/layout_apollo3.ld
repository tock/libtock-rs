/* Layout for the Apollo3 MCU, used by the examples in this repository. */

MEMORY {
  /* The application region is 64 bytes (0x40) */
  FLASH (rx) : ORIGIN = 0x00040040, LENGTH = 0x0005FFC0
  SRAM (rwx) : ORIGIN = 0x10002000, LENGTH = 0x2000
}

MPU_MIN_ALIGN = 8K;

INCLUDE layout_generic.ld
