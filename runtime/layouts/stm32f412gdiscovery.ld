/* Layout for the stm32f412gdiscovery board, usable by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  FLASH (X) : ORIGIN = 0x08030000 + TBF_HEADER_SIZE, LENGTH = 256K - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x20004000                  , LENGTH = 112K
}

INCLUDE libtock_layout.ld
