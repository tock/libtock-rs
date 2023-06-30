/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x80;

FLASH_START = 0x20030000;
FLASH_LENGTH = 32M;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x10004130;
RAM_LENGTH = 512K;

INCLUDE libtock_layout.ld
