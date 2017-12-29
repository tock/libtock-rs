#!/usr/bin/env bash

set -eux

cargo install -f xargo
cargo install -f --git https://github.com/helena-project/tock.git elf2tbf