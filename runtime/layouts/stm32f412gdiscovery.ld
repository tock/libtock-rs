/* Layout for the stm32f412gdiscovery board, usable by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x08030000;
FLASH_LENGTH = 256K;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x20004000;
RAM_LENGTH = 112K;

INCLUDE libtock_layout.ld
