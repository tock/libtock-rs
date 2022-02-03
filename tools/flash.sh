#!/usr/bin/env bash

TBF_HEADER=64
KERNEL_VERSION=""

if [ ! -z $LIBTOCK_PLATFORM ]; then
    # we are using libtock2
	PLATFORM=$LIBTOCK_PLATFORM
	TBF_HEADER=72
    KERNEL_VERSION="--kernel-major 2 --kernel-minor 0"
fi

set -eux

artifact="$(basename $1)"
rust_target_folder="$(cd $(dirname $1)/../.. && pwd -P)"
if [ -z $APP_HEAP_SIZE ]; then
	echo "Set APP_HEAP_SIZE to a value"
	exit 1
fi

if [ -z $KERNEL_HEAP_SIZE ]; then
	echo "Set KERNEL_HEAP_SIZE to a value"
	exit 1
fi

case "${PLATFORM}" in
    "apollo3")
        tockloader_flags=""
        binary_name=cortex-m4.elf
        tockload=n
        ;;
    "esp32-c3-devkitM-1")
        tockloader_flags=""
        binary_name=rv32imac.elf
        tockload=n
        ;;
    "microbit_v2")
        tockloader_flags="--bundle-apps"
        binary_name=cortex-m4.elf
        tockload=y
        ;;
    "nucleo_f429zi"|"nucleo_f446re")
        tockloader_flags=""
        binary_name=cortex-m4.elf
        tockload=n
        ;;
    "nrf52"|"nrf52840")
        tockloader_flags="--jlink --arch cortex-m4 --board nrf52dk --jtag-device nrf52"
        binary_name=cortex-m4.elf
        tockload=y
        ;;
    "hail")
        tockloader_flags=""
        binary_name=cortex-m4.elf
        tockload=y
        ;;
    "hifive1")
        tockloader_flags=""
        binary_name=rv32imac.elf
        tockload=n
        ;;
    "imxrt1050")
        tockloader_flags=""
        binary_name=cortex-m7.elf
        tockload=n
        ;;
    "msp432")
        tockloader_flags=""
        binary_name=cortex-m4.elf
        tockload=n
        ;;
    "opentitan")
        tockloader_flags=""
        binary_name=rv32imc.elf
        tockload=n
        ;;
    *)
        echo "Unknown platform \"${PLATFORM}\""
        exit 1
        ;;
esac

libtock_target_path="${rust_target_folder}/tab/${PLATFORM}/${artifact}"
elf_file_name="${libtock_target_path}/${binary_name}"
tab_file_name="${libtock_target_path}.tab"

mkdir -p "${libtock_target_path}"
cp "$1" "${elf_file_name}"

STACK_SIZE=$(nm --print-size --size-sort --radix=d "${elf_file_name}" | grep STACK_MEMORY | cut -d " " -f 2)

elf2tab -n "${artifact}" -o "${tab_file_name}" "${elf_file_name}" --stack ${STACK_SIZE} --app-heap $APP_HEAP_SIZE --kernel-heap $KERNEL_HEAP_SIZE --protected-region-size=$TBF_HEADER $KERNEL_VERSION

if [ $tockload == "n" ]; then
	echo "Skipping flashing for platform \"${PLATFORM}\""
	exit 0
fi

if ! [ -x "$(command -v tockloader)" ]; then
    echo "Skipping flashing as tockloader isn't installed"
    exit 0
fi

tockloader uninstall ${tockloader_flags} || true
tockloader install ${tockloader_flags} "${tab_file_name}"
