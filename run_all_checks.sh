#!/usr/bin/env bash

set -eux

export PLATFORM=nrf52 # The specific platform doesn't matter for tests

cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets
./build_examples.sh
