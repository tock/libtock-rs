/* Layout for the nRF52840-DK, usable by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x00030000;
FLASH_LENGTH = 0x000D0000;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x20008000;
RAM_LENGTH = 46K;

INCLUDE libtock_layout.ld
