/* Layout for the iMX.RT1050 board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x63002000;
FLASH_LENGTH = 0x1000000;

RAM_START = 0x20004000;
RAM_LENGTH = 112K;

INCLUDE libtock_layout.ld
