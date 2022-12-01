/* Layout for the micro:bit v2 board, used by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x00040000, LENGTH = 256K
  RAM (W) : ORIGIN = 0x20004000, LENGTH = 112K
}

TBF_HEADER_SIZE = 0x60;
INCLUDE libtock_layout.ld
