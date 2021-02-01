#!/bin/bash

set -e

# Switch into the directory this script is in, in case it was run from another
# location.
cd "$(dirname "$0")"

# Our CI runs in GitHub Actions' Ubuntu 20.04 image. The only RISC-V toolchain
# in Ubuntu 20.04's repositories is for riscv64-linux-gnu. Fortunately, this
# toolchain can output 32-bit RISC-V assembly using the -march= option, and the
# fact it is targeted at GNU/Linux doesn't matter for our short handwritten
# assembly segment.
#
# Although we also support rv32imac targets, we do not need to separately
# assemble for it, as asm_riscv32.S does not use atomic instructions.
riscv64-linux-gnu-as -march=rv32imc asm_riscv32.S -o riscv32.o
# For some reason, riscv64-linux-gnu-as includes local symbols in its output.
# This pollutes the output of `objdump`, making debugging more difficult. This
# strips the extra symbols to keep the disassembly readable.
riscv64-linux-gnu-strip -K start -K rust_start riscv32.o

# Remove the archive file in case there is something unexpected in it (so that
# issues cannot persist across calls to this script).
rm -f libriscv32.a
# c == do not complain if archive needs to be created
# r == insert or replace file in archive
riscv64-linux-gnu-ar cr libriscv32.a riscv32.o
# Remove riscv32.o as it is an intermediate build artifact.
rm riscv32.o
