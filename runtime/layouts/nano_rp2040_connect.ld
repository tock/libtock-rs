/* Layout for the Raspberry Pi Pico board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x10020000;
FLASH_LENGTH = 256K;

RAM_START = 0x20004000;
RAM_LENGTH = 248K;

INCLUDE libtock_layout.ld
