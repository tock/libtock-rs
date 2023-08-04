/* Layout for the Imix board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x00040000;
FLASH_LENGTH = 0x00040000;

RAM_START = 0x20008000;
RAM_LENGTH = 62K;

INCLUDE libtock_layout.ld
