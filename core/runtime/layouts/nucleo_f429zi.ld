/* Layout for the Nucleo F429zi, used by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x08040000, LENGTH = 255K
  RAM   (W) : ORIGIN = 0x20004000, LENGTH = 112K
}

TBF_HEADER_SIZE = 0x40;
INCLUDE libtock_layout.ld
