/* Layout for the esp32c3 DevKit M1 board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

FLASH_START = 0x403B0000;
FLASH_LENGTH = 0x30000;

/* Note that the SRAM address may need to be changed depending on
 * the kernel binary, check for the actual address of APP_MEMORY!
 */
RAM_START = 0x3FCA1800;
RAM_LENGTH = 0x2E800;

INCLUDE libtock_layout.ld
