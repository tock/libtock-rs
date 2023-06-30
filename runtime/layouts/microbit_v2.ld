/* Layout for the micro:bit v2 board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x00040000;
FLASH_LENGTH = 256K;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x20004000;
RAM_LENGTH = 112K;

INCLUDE libtock_layout.ld
