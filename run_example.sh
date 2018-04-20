#!/usr/bin/env bash

# Examples only run on a nRF52-DK board

set -eux

tab_file_name=metadata.toml
elf_file_name=cortex-m4.elf
bin_file_name=cortex-m4.bin

cargo build --release --target=thumbv7em-none-eabi --example "$1"
cp target/thumbv7em-none-eabi/release/examples/"$1" "target/$elf_file_name"
cargo run --manifest-path tock/userland/tools/elf2tbf/Cargo.toml -- -n "$1" -o "target/$bin_file_name" "target/$elf_file_name"

echo "tab-version = 1" > "target/$tab_file_name"
echo "name = \"$1\"" >> "target/$tab_file_name"
echo "only-for-boards = \"\"" >> "target/$tab_file_name"
echo "build-date = $(date "+%Y-%m-%dT%H:%M:%SZ")" >> "target/$tab_file_name"

out_file_name="$1".tab
tar -C target -cf "target/$out_file_name" "$bin_file_name" "$tab_file_name"

if [ "$#" -ge "2" ]
then
    if [ "$2" = "--dont-clear-apps" ]
    then
        echo "do not delete apps from board."
    else
        tockloader uninstall --jtag --arch cortex-m4 --board nrf52-dk --jtag-device nrf52 --app-address 0x20000 || true
    fi
else
    tockloader uninstall --jtag --arch cortex-m4 --board nrf52-dk --jtag-device nrf52 --app-address 0x20000 || true
fi
tockloader install --jtag --arch cortex-m4 --board nrf52-dk --jtag-device nrf52 --app-address 0x20000 "target/$out_file_name"
