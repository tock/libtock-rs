/* Layout for the esp32c3 DevKit M1 board, used by the examples in this repository. */

MEMORY {
  /* Note that the SRAM address may need to be changed depending on
   * the kernel binary, check for the actual address of APP_MEMORY!
   */
  FLASH (X) : ORIGIN = 0x403B0000, LENGTH = 0x30000
  RAM   (W) : ORIGIN = 0x3FCA1800, LENGTH = 0x2E800
}

TBF_HEADER_SIZE = 0x48;
INCLUDE libtock_layout.ld
