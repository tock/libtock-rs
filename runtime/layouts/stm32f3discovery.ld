/* Layout for the stm32f3discovery board, usable by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x08020000;
FLASH_LENGTH = 0x00020000;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x20004000;
RAM_LENGTH = 48K;

INCLUDE libtock_layout.ld
