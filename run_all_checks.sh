#!/usr/bin/env bash

set -eux

cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets
./build_examples.sh