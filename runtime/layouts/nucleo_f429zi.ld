/* Layout for the Nucleo F429zi, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x08040000 + TBF_HEADER_SIZE, LENGTH = 255K - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x20004000                  , LENGTH = 112K
}

INCLUDE libtock_layout.ld
