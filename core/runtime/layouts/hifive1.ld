/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

MEMORY {
  /* Note that the SRAM address may need to be changed depending on
   * the kernel binary, check for the actual address of APP_MEMORY!
   */
  FLASH (X) : ORIGIN = 0x20040000, LENGTH = 32M
  RAM   (W) : ORIGIN = 0x80002400, LENGTH = 0x1C00
}

TBF_HEADER_SIZE = 0x40;
INCLUDE libtock_layout.ld
