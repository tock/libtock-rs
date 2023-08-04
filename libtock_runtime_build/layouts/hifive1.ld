/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x20040000;
FLASH_LENGTH = 32M;

RAM_START = 0x80003000;
RAM_LENGTH = 0x1000;

INCLUDE libtock_layout.ld
