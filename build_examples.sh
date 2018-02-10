#!/usr/bin/env bash

set -eux

cargo run --manifest-path xargo/Cargo.toml -- build --target=thumbv7em-tock-eabi --examples
