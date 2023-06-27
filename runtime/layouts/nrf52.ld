/* Layout for the nRF52-DK, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x00030000 + TBF_HEADER_SIZE, LENGTH = 0x00060000 - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x20004000                  , LENGTH = 62K
}

INCLUDE libtock_layout.ld
