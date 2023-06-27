/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

TBF_HEADER_SIZE = 0x80;

MEMORY {
  /* Note that the SRAM address may need to be changed depending on
   * the kernel binary, check for the actual address of APP_MEMORY!
   */
  FLASH (X) : ORIGIN = 0x20030000 + TBF_HEADER_SIZE, LENGTH = 32M - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = 0x10004130                  , LENGTH = 512K
}

INCLUDE libtock_layout.ld
