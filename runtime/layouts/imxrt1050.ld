/* Layout for the iMX.RT1050 board, used by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x63002000, LENGTH = 0x1000000
  RAM   (W) : ORIGIN = 0x20004000, LENGTH = 112K
}

TBF_HEADER_SIZE = 0x40;
INCLUDE libtock_layout.ld
