/* Layout for the micro:bit v2 board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x00080000 + TBF_HEADER_SIZE, LENGTH = 512K - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x20006000                  , LENGTH = 216K
}

INCLUDE libtock_layout.ld
