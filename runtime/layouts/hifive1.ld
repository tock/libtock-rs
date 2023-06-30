/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x20040000;
FLASH_LENGTH = 32M;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x80002800;
RAM_LENGTH = 0x1800;

INCLUDE libtock_layout.ld
