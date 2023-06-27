/* Layout for the Imix board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x00040000 + TBF_HEADER_SIZE, LENGTH = 0x00040000 - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x20008000                  , LENGTH = 62K
}

INCLUDE libtock_layout.ld
