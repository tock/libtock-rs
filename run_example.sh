#!/usr/bin/env bash

# Examples only run on a nRF52-DK board

set -eux

cargo build --release --target=thumbv7em-none-eabi --example "$1"


elf_file_name="target/tab/$1/cortex-m4.elf"
tab_file_name="target/tab/$1.tab"

# Default value for nRF52-DK
tockloader_flags="--jlink --arch cortex-m4 --board nrf52dk --jtag-device nrf52 --app-address 0x20000"

hail_defined=${hail:-}
if [ -n "$hail_defined" ]
then
    tockloader_flags=""
fi

mkdir -p "target/tab/$1"
cp "target/thumbv7em-none-eabi/release/examples/$1" "$elf_file_name"

elf2tab -n "$1" -o "$tab_file_name" "$elf_file_name" --stack 2048 --app-heap 1024 --kernel-heap 1024 --protected-region-size=64

if [ "$#" -ge "2" ]
then
    if [ "$2" = "--dont-clear-apps" ]
    then
        echo "do not delete apps from board."
    else
        tockloader uninstall $tockloader_flags || true
    fi
else
    tockloader uninstall $tockloader_flags || true
fi
tockloader install $tockloader_flags "$tab_file_name"
