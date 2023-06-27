/* Layout for the stm32f3discovery board, usable by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x08020000 + TBF_HEADER_SIZE, LENGTH = 0x00020000 - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x20004000                  , LENGTH = 48K
}

INCLUDE libtock_layout.ld
