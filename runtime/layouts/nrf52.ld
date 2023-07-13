/* Layout for the nRF52-DK, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x00030000;
FLASH_LENGTH = 0x00060000;

RAM_START = 0x20004000;
RAM_LENGTH = 62K;

INCLUDE libtock_layout.ld
