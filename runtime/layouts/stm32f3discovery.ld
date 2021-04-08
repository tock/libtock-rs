/* Layout for the stm32f3discovery board, usable by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x08020000, LENGTH = 0x00020000
  RAM   (W) : ORIGIN = 0x20004000, LENGTH = 48K
}

TBF_HEADER_SIZE = 0x40;
INCLUDE libtock_layout.ld
