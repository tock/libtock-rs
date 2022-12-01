/* Layout for the Imix board, used by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x00040000, LENGTH = 0x00040000
  RAM   (W) : ORIGIN = 0x20008000, LENGTH = 62K
}

TBF_HEADER_SIZE = 0x60;
INCLUDE libtock_layout.ld
