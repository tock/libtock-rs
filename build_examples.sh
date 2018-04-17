#!/usr/bin/env bash

set -eux

cargo build --release --target=thumbv7em-none-eabi --examples
