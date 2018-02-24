#!/usr/bin/env bash

set -eux
export RUST_TARGET_PATH=`pwd`
export CARGO_INCREMENTAL=0

cargo run --manifest-path xargo/Cargo.toml -- build --release --target=thumbv7em-tock-eabi --examples
