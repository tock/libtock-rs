#!/usr/bin/env bash

# Examples only run on a nRF52-DK board

set -eux

cargo build --release --target=thumbv7em-none-eabi --example "$1"


elf_file_name="$(pwd)/target/tab/$1/cortex-m4.elf"
tab_file_name="$(pwd)/target/tab/$1.tab"

mkdir -p "target/tab/$1"
cp "target/thumbv7em-none-eabi/release/examples/$1" "$elf_file_name"

pushd ../elf2tab
cargo run -- -v -n "$1" -o "$tab_file_name" "$elf_file_name" --stack 2048 --app-heap 2048 --kernel-heap 1024
popd

if [ "$#" -ge "2" ]
then
    if [ "$2" = "--dont-clear-apps" ]
    then
        echo "do not delete apps from board."
    else
        tockloader uninstall --jlink --arch cortex-m4 --board nrf52dk --jtag-device nrf52 --app-address 0x20000 || true
    fi
else
    tockloader uninstall --jlink --arch cortex-m4 --board nrf52dk --jtag-device nrf52 --app-address 0x20000 || true
fi
tockloader install --jlink --arch cortex-m4 --board nrf52dk --jtag-device nrf52 --app-address 0x20000 "$tab_file_name"
