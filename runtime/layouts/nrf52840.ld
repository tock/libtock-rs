/* Layout for the nRF52840-DK, usable by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x00030000, LENGTH = 0x000D0000
  RAM   (W) : ORIGIN = 0x20010000, LENGTH = 46K
}

TBF_HEADER_SIZE = 0x48;
INCLUDE libtock_layout.ld
