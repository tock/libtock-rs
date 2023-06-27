/* Layout for the iMX.RT1050 board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x63002000 + TBF_HEADER_SIZE, LENGTH = 0x1000000 - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x20004000                  , LENGTH = 112K
}

INCLUDE libtock_layout.ld
