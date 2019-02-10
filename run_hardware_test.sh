#!/usr/bin/env bash

# Tests only run on a nRF52-DK board

set -eux

yes 0|tockloader uninstall --jlink --arch cortex-m4 --board nrf52dk --jtag-device nrf52 --app-address 0x20000 || true

./run_example.sh hardware_test --dont-clear-apps
