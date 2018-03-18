#!/usr/bin/env bash

set -eux

RUST_TARGET_PATH=$(pwd) cargo run --manifest-path xargo/Cargo.toml -- build --release --target=thumbv7em-tock-eabi --examples
