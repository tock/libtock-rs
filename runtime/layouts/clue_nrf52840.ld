/* Layout for the micro:bit v2 board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x00080000;
FLASH_LENGTH = 512K;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x20006000;
RAM_LENGTH = 216K;

INCLUDE libtock_layout.ld
