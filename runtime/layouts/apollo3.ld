/* Layout for the Apollo3 MCU, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x00040000 + TBF_HEADER_SIZE, LENGTH = 0x00060000 - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x10002000                  , LENGTH = 0x2000
}

INCLUDE libtock_layout.ld
