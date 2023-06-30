/* Layout for the Raspberry Pi Pico board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x10020000;
FLASH_LENGTH = 256K;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x20004000;
RAM_LENGTH = 248K;

INCLUDE libtock_layout.ld
