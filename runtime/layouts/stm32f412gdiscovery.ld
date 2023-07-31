/* Layout for the stm32f412gdiscovery board, usable by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x08030000;
FLASH_LENGTH = 256K;

RAM_START = 0x20004000;
RAM_LENGTH = 112K;

INCLUDE libtock_layout.ld
