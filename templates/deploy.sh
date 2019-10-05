#!/usr/bin/env bash

set -eux

cargo build --release --target=thumbv7em-none-eabi


elf_file_name="target/tab/cortex-m4.elf"
tab_file_name="target/tab/main.tab"

tockloader_flags=""

mkdir -p "target/tab"
cp "target/thumbv7em-none-eabi/release/<application>" "$elf_file_name"

elf2tab -n "main" -o "$tab_file_name" "$elf_file_name" --stack 2048 --app-heap 1024 --kernel-heap 1024 --protected-region-size=64

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
