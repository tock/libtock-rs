/* Layout for the nRF52840-DK, usable by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x00030000 + TBF_HEADER_SIZE, LENGTH = 0x000D0000 - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x20008000                  , LENGTH = 46K
}

INCLUDE libtock_layout.ld
