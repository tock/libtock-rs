/* Layout for the Raspberry Pi Pico board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x10020000 + TBF_HEADER_SIZE, LENGTH = 256K - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x20004000                  , LENGTH = 248K
}

INCLUDE libtock_layout.ld
