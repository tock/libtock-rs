/* Layout for the micro:bit v2 board, used by the examples in this repository. */

MEMORY {
  FLASH (rx) : ORIGIN = 0x00040000, LENGTH = 256K
  RAM (rwx) : ORIGIN = 0x20004000, LENGTH = 112K
}

MPU_MIN_ALIGN = 8K;

TBF_HEADER_SIZE = 0x48;
INCLUDE libtock_layout.ld
