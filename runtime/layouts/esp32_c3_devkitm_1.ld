/* Layout for the esp32c3 DevKit M1 board, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  /* Note that the SRAM address may need to be changed depending on
   * the kernel binary, check for the actual address of APP_MEMORY!
   */
  FLASH (X) : ORIGIN = 0x403B0000 + TBF_HEADER_SIZE, LENGTH = 0x30000 - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x3FCA1800                  , LENGTH = 0x2E800
}

INCLUDE libtock_layout.ld
