/* Layout for the Apollo3 MCU, used by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x00040000, LENGTH = 0x00060000
  RAM   (W) : ORIGIN = 0x10002000, LENGTH = 0x2000
}

TBF_HEADER_SIZE = 0x40;
INCLUDE libtock_layout.ld
