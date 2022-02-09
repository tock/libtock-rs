/* Layout for the Raspberry Pi Pico board, used by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x10020000, LENGTH = 256K
  RAM (W) : ORIGIN = 0x20004000, LENGTH = 248K
}

TBF_HEADER_SIZE = 0x48;
INCLUDE libtock_layout.ld
