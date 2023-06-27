/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x60;

MEMORY {
  /* Note that the SRAM address may need to be changed depending on
   * the kernel binary, check for the actual address of APP_MEMORY!
   */
  FLASH (X) : ORIGIN = 0x20040000 + TBF_HEADER_SIZE, LENGTH = 32M - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x80002800                  , LENGTH = 0x1800
}

INCLUDE libtock_layout.ld
