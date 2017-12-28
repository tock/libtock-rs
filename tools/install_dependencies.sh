#!/usr/bin/env bash

set -eux

cargo install -f xargo
cargo install -f --git https://github.com/helena-project/tock.git --rev ba0e53384f1c037e7cc13eec2ada3bf59820f98c elf2tbf