#!/usr/bin/env bash

set -eux

cargo install -f --git https://github.com/japaric/xargo --vers 0.3.10
cargo install -f --git https://github.com/helena-project/tock.git elf2tbf --rev 09ed0a57c58d84595910b410494240ed995577ab