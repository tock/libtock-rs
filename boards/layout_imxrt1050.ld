/* Layout for the iMX.RT1050 board, used by the examples in this repository. */

MEMORY {
  /* The application region is 64 bytes (0x40) */
  FLASH (rx) : ORIGIN = 0x63002040, LENGTH = 0xFFFFC0
  SRAM (rwx) : ORIGIN = 0x20004000, LENGTH = 112K
}

MPU_MIN_ALIGN = 8K;

INCLUDE layout_generic.ld
