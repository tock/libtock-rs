#!/usr/bin/env bash

set -eux

PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --examples
PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --examples --features=alloc
PLATFORM=riscv32 cargo build --release --target=riscv32imc-unknown-none-elf --examples # Important for testing: This target does not support atomics
