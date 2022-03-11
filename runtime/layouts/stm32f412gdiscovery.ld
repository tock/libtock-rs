/* Layout for the stm32f412gdiscovery board, usable by the examples in this repository. */

MEMORY {
  FLASH (X) : ORIGIN = 0x08030000, LENGTH = 256K
  RAM   (W) : ORIGIN = 0x20004000, LENGTH = 112K
}

TBF_HEADER_SIZE = 0x48;
INCLUDE libtock_layout.ld
