/* Layout for the esp32c3 DevKit M1 board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x403B0000;
FLASH_LENGTH = 0x30000;

RAM_START = 0x3FCA2000;
RAM_LENGTH = 0x2E000;

INCLUDE libtock_layout.ld
