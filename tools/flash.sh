#!/usr/bin/env bash

set -eu

get_drivers() {
    # List all of the drivers in the IR output
    driver_output=$(cat ${rust_target_folder}/release/examples/${artifact}*.ll | grep "^%\"libtock::")
    # Remove all of the parts we don't need to get a nice list
    used_drivers=$(echo "${driver_output}" | cut -d ":" -f 3 | sort -u)

    for driver in ${used_drivers}; do
        # Get the full official kernel name
        cat tock/capsules/src/driver.rs | grep -i ${driver} | tr -s ' ' | cut -d ' ' -f 2
    done
}

get_driver_numbers_and_perms() {
    # Lookup in the kernel what driver number is used (in hex).
    driver_nums=$(for driver in ${1}; do
        cat tock/capsules/src/driver.rs | grep -i ${driver} | tr -s ' ' | cut -d ' ' -f 4 | cut -d , -f 1 | cut -d x -f 2
    done)

    for driver in ${driver_nums}; do
        # Convert the number to a decimal value
        driver_base_ten=$(echo "obase=10; ibase=16; ${driver}" | bc)

        # We now know the driver.
        # We don't have an easy way to figure out what commands we are using.
        # So instead we just allow all of them by default.

        # Get a list of all commands in libtock-s
        name=$(cat tock/capsules/src/driver.rs | grep ${driver} | tr -s ' ' | cut -d ' ' -f 2)
        lower_name=$(echo "$name" | awk '{print tolower($0)}')

        # Get all of the command numbers
        command_nums=$(sed -n '/mod command_nr {/,/}/p' src/${lower_name}.rs | grep usize | tr -s ' ' | cut -d ' ' -f 7 | cut -d ';' -f 1)

        for num in ${command_nums}; do
            echo ${driver_base_ten},${num}
        done
    done
}

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

drivers=$(get_drivers)

echo "This app uses the following drivers"
echo ${drivers}

perms=$(get_driver_numbers_and_perms "${drivers}")

elf2tab -n "${artifact}" -o "${tab_file_name}" "${elf_file_name}" \
        --stack ${STACK_SIZE} --app-heap $APP_HEAP_SIZE --kernel-heap $KERNEL_HEAP_SIZE \
        --protected-region-size=64 --permissions ${perms} --verbose

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
