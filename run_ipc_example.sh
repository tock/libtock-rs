#!/usr/bin/env bash

# Examples only run on a NRF52DK board

set -eux

yes 0|tockloader uninstall --jtag --arch cortex-m4 --board nrf52-dk --jtag-device nrf52 --app-address 0x20000 || true

./run_example.sh ipcclient --dont-clear-apps
./run_example.sh ipcserver --dont-clear-apps
