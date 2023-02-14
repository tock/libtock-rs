/* Layout for the Raspberry Pi Pico board, used by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x10040000, LENGTH = 256K
  RAM (W) : ORIGIN = 0x20011800, LENGTH = 194K
}

TBF_HEADER_SIZE = 0x60;
INCLUDE libtock_layout.ld
