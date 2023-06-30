/* Layout for the Hail board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x00030000;
FLASH_LENGTH = 0x00040000;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x20008000;
RAM_LENGTH = 62K;

INCLUDE libtock_layout.ld
