#!/usr/bin/env bash

set -eux

xargo build --target=thumbv7em-tock-eabi --examples