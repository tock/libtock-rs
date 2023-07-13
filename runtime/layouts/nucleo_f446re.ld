/* Layout for the Nucleo F446re, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x08040000;
FLASH_LENGTH = 255K;

RAM_START = 0x20004000;
RAM_LENGTH = 176K;

INCLUDE libtock_layout.ld
