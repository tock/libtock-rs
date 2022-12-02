/* Layout for the micro:bit v2 board, used by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x00080000, LENGTH = 512K
  RAM (W) : ORIGIN = 0x20006000, LENGTH = 216K
}

TBF_HEADER_SIZE = 0x60;
INCLUDE libtock_layout.ld
