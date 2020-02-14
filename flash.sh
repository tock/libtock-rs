#!/usr/bin/env bash

set -eux

artifact="$(basename $1)"
rust_target_folder="$(readlink -f $(dirname $1)/../..)"
libtock_target_path="${rust_target_folder}/tab/${PLATFORM}/${artifact}"
elf_file_name="${libtock_target_path}/cortex-m4.elf"
tab_file_name="${libtock_target_path}.tab"

mkdir -p "${libtock_target_path}"
cp "$1" "${elf_file_name}"

elf2tab -n "${artifact}" -o "${tab_file_name}" "${elf_file_name}" --stack 2048 --app-heap 1024 --kernel-heap 1024 --protected-region-size=64

case "${PLATFORM}" in
    "nrf52"|"nrf52840")
        tockloader_flags="--jlink --arch cortex-m4 --board nrf52dk --jtag-device nrf52"
        ;;
    "hail")
        tockloader_flags=""
        ;;
    *)
        echo "Tockloader flags unknown for platform \"${PLATFORM}\""
        exit 1
        ;;
esac

tockloader uninstall ${tockloader_flags} || true
tockloader install ${tockloader_flags} "${tab_file_name}"
