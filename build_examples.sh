#!/usr/bin/env bash

set -eux

cargo build --release --target=thumbv7em-none-eabi --examples
cargo build --release --target=riscv32imc-unknown-none-elf --examples # Important for tests: This target does not support atomics
