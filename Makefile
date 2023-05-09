# By default, let's print out some help
.PHONY: usage
usage:
	@echo "$$(tput bold)Welcome to libtock-rs!$$(tput sgr0)"
	@echo
	@echo "First things first, if you haven't yet, check out Tocks's doc/Getting_Started."
	@echo "After that read the README from libtock-rs"
	@echo "You'll need to install a few requirements before we get going."
	@echo
	@echo "The next step is to choose a board to build Tock for. Mainline"
	@echo "libtock-rs currently includes support for the following platforms:"
	@echo " - hail"
	@echo " - nrf52840"
	@echo " - microbit_v2"
	@echo " - nucleo_f429zi"
	@echo " - nucleo_f446re"
	@echo " - opentitan"
	@echo " - hifive1"
	@echo " - nrf52"
	@echo " - imxrt1050"
	@echo " - apollo3"
	@echo " - stm32f3discovery"
	@echo " - stm32f412gdiscovery"
	@echo " - esp32_c3_devkitm_1"
	@echo " - clue_nrf52840"
	@echo
	@echo "Run 'make setup' to setup Rust to build libtock-rs."
	@echo "Run 'make <board> EXAMPLE=<>' to build EXAMPLE for that board."
	@echo "Run 'make flash-<board> EXAMPLE=<>' to flash EXAMPLE to a tockloader-supported board."
	@echo "Run 'make qemu-example EXAMPLE=<>' to run EXAMPLE in QEMU"
	@echo "Run 'make test' to test any local changes you have made"
	@echo "Run 'make print-sizes' to print size data for the example binaries"

ifdef FEATURES
features=--features=$(FEATURES)
endif

ifndef DEBUG
release=--release
endif

.PHONY: setup
setup: setup-qemu
	cargo install elf2tab
	cargo miri setup
	rustup target add --toolchain stable thumbv7em-none-eabi

# Sets up QEMU in the tock/ directory. We use Tock's QEMU which may contain
# patches to better support boards that Tock supports.
.PHONY: setup-qemu
setup-qemu:
	CI=true $(MAKE) -C tock ci-setup-qemu

# Builds a Tock 2.0 kernel for the HiFive board for use by QEMU tests.
.PHONY: kernel-hifive
kernel-hifive:
	$(MAKE) -C tock/boards/hifive1 \
		$(CURDIR)/tock/target/riscv32imac-unknown-none-elf/release/hifive1.elf

# Builds a Tock kernel for the OpenTitan board on the cw310 FPGA for use by QEMU
# tests.
.PHONY: kernel-opentitan
kernel-opentitan:
	CARGO_TARGET_RISCV32IMC_UNKNOWN_NONE_ELF_RUNNER="[]" \
		$(MAKE) -C tock/boards/opentitan/earlgrey-cw310 \
		$(CURDIR)/tock/target/riscv32imc-unknown-none-elf/release/earlgrey-cw310.elf

# Prints out the sizes of the example binaries.
.PHONY: print-sizes
print-sizes: examples
	cargo run --release -p print_sizes

# Runs a libtock example in QEMU on a simulated HiFive board.
.PHONY: qemu-example
qemu-example: kernel-hifive
	LIBTOCK_PLATFORM="hifive1" cargo run --example "$(EXAMPLE)" -p libtock \
		--release --target=riscv32imac-unknown-none-elf -- --deploy qemu

# Build the examples on both a RISC-V target and an ARM target. We pick
# opentitan as the RISC-V target because it lacks atomics.
.PHONY: examples
examples:
	LIBTOCK_PLATFORM=nrf52 cargo build --examples --release \
		--target=thumbv7em-none-eabi
	LIBTOCK_PLATFORM=opentitan cargo build --examples --release \
		--target=riscv32imc-unknown-none-elf

# Arguments to pass to cargo to exclude crates that require a Tock runtime.
# This is largely libtock_runtime and crates that depend on libtock_runtime.
# Used when we need to build a crate for the host OS, as libtock_runtime only
# supports running on Tock.
EXCLUDE_RUNTIME := --exclude libtock --exclude libtock_runtime \
	--exclude libtock_debug_panic --exclude libtock_small_panic

# Arguments to pass to cargo to exclude crates that cannot be tested by Miri. In
# addition to excluding libtock_runtime, Miri also cannot test proc macro crates
# (and in fact will generate broken data that causes cargo test to fail).
EXCLUDE_MIRI := $(EXCLUDE_RUNTIME) --exclude ufmt-macros

# Arguments to pass to cargo to exclude `std` and crates that depend on it. Used
# when we build a crate for an embedded target, as those targets lack `std`.
EXCLUDE_STD := --exclude libtock_unittest --exclude print_sizes \
               --exclude runner --exclude syscalls_tests

# Currently, all of our crates should build with a stable toolchain. This
# verifies our crates don't depend on unstable features by using cargo check. We
# specify a different target directory so this doesn't flush the cargo cache of
# the primary toolchain.
.PHONY: test-stable
test-stable:
	CARGO_TARGET_DIR="target/stable-toolchain" cargo +stable check --workspace \
		$(EXCLUDE_RUNTIME)
	CARGO_TARGET_DIR="target/stable-toolchain" LIBTOCK_PLATFORM=nrf52 cargo \
		+stable check $(EXCLUDE_STD) --target=thumbv7em-none-eabi --workspace

.PHONY: test
test: examples test-stable
	cargo test $(EXCLUDE_RUNTIME) --workspace
	LIBTOCK_PLATFORM=nrf52 cargo fmt --all -- --check
	cargo clippy --all-targets $(EXCLUDE_RUNTIME) --workspace
	LIBTOCK_PLATFORM=nrf52 cargo clippy $(EXCLUDE_STD) \
		--target=thumbv7em-none-eabi --workspace
	LIBTOCK_PLATFORM=hifive1 cargo clippy $(EXCLUDE_STD) \
		--target=riscv32imac-unknown-none-elf --workspace
	MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-symbolic-alignment-check" \
		cargo miri test $(EXCLUDE_MIRI) --workspace
	echo '[ SUCCESS ] libtock-rs tests pass'

.PHONY: apollo3
apollo3:
	LIBTOCK_PLATFORM=apollo3 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/apollo3
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/apollo3

.PHONY: esp32_c3_devkitm_1
esp32_c3_devkitm_1:
	LIBTOCK_PLATFORM=esp32_c3_devkitm_1 cargo run --example $(EXAMPLE) $(features) \
		--target=riscv32imc-unknown-none-elf $(release)
	mkdir -p target/tbf/esp32_c3_devkitm_1
	cp target/riscv32imc-unknown-none-elf/release/examples/$(EXAMPLE).tab \
		target/riscv32imc-unknown-none-elf/release/examples/$(EXAMPLE).tbf \
		target/tbf/esp32_c3_devkitm_1

.PHONY: hail
hail:
	LIBTOCK_PLATFORM=hail cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/hail
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/hail

.PHONY: flash-hail
flash-hail:
	LIBTOCK_PLATFORM=hail cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release) -- --deploy=tockloader

.PHONY: imix
imix:
	LIBTOCK_PLATFORM=imix cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/imix
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/imix

.PHONY: flash-imix
flash-imix:
	LIBTOCK_PLATFORM=imix cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release) -- --deploy=tockloader


.PHONY: microbit_v2
microbit_v2:
	LIBTOCK_PLATFORM=microbit_v2 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/microbit_v2
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/microbit_v2

.PHONY: flash-microbit_v2
flash-microbit_v2:
	LIBTOCK_PLATFORM=microbit_v2 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release) -- --deploy=tockloader

.PHONY: nucleo_f429zi
nucleo_f429zi:
	LIBTOCK_PLATFORM=nucleo_f429zi cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/nucleo_f429zi
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/nucleo_f429zi

.PHONY: nucleo_f446re
nucleo_f446re:
	LIBTOCK_PLATFORM=nucleo_f446re cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/nucleo_f446re
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/nucleo_f446re

.PHONY: nrf52840
nrf52840:
	LIBTOCK_PLATFORM=nrf52840 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/nrf52840
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/nrf52840

.PHONY: flash-nrf52840
flash-nrf52840:
	LIBTOCK_PLATFORM=nrf52840 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release) -- --deploy=tockloader

.PHONY: raspberry_pi_pico
raspberry_pi_pico:
	LIBTOCK_PLATFORM=raspberry_pi_pico cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv6m-none-eabi $(release)
	mkdir -p target/tbf/raspberry_pi_pico
	cp target/thumbv6m-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv6m-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/raspberry_pi_pico

.PHONY: nano_rp2040_connect
nano_rp2040_connect:
	LIBTOCK_PLATFORM=nano_rp2040_connect cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv6m-none-eabi $(release)
	mkdir -p target/tbf/nano_rp2040_connect
	cp target/thumbv6m-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv6m-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/nano_rp2040_connect

.PHONY: stm32f3discovery
stm32f3discovery:
	LIBTOCK_PLATFORM=stm32f3discovery cargo run --example $(EXAMPLE) \
		$(features) --target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/stm32f3discovery
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/stm32f3discovery

.PHONY: stm32f412gdiscovery
stm32f412gdiscovery:
	LIBTOCK_PLATFORM=stm32f412gdiscovery cargo run --example $(EXAMPLE) \
		$(features) --target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/stm32f412gdiscovery
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/stm32f412gdiscovery

.PHONY: opentitan
opentitan:
	LIBTOCK_PLATFORM=opentitan cargo run --example $(EXAMPLE) $(features) \
		--target=riscv32imc-unknown-none-elf $(release)
	mkdir -p target/tbf/opentitan
	cp target/riscv32imc-unknown-none-elf/release/examples/$(EXAMPLE).tab \
		target/riscv32imc-unknown-none-elf/release/examples/$(EXAMPLE).tbf \
		target/tbf/opentitan

.PHONY: hifive1
hifive1:
	LIBTOCK_PLATFORM=hifive1 cargo run --example $(EXAMPLE) $(features) \
		--target=riscv32imac-unknown-none-elf $(release)
	mkdir -p target/tbf/hifive1
	cp target/riscv32imac-unknown-none-elf/release/examples/$(EXAMPLE).tab \
		target/riscv32imac-unknown-none-elf/release/examples/$(EXAMPLE).tbf \
		target/tbf/hifive1

.PHONY: nrf52
nrf52:
	LIBTOCK_PLATFORM=nrf52 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/nrf52
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/nrf52

.PHONY: flash-nrf52
flash-nrf52:
	LIBTOCK_PLATFORM=nrf52 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release) -- --deploy=tockloader

.PHONY: imxrt1050
imxrt1050:
	LIBTOCK_PLATFORM=imxrt1050 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/imxrt1050
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/imxrt1050

.PHONY: msp432
msp432:
	LIBTOCK_PLATFORM=msp432 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/msp432
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/msp432

.PHONY: clue_nrf52840
clue_nrf52840:
	LIBTOCK_PLATFORM=clue_nrf52840 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release)
	mkdir -p target/tbf/clue_nrf52840
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tab \
		target/thumbv7em-none-eabi/release/examples/$(EXAMPLE).tbf \
		target/tbf/clue_nrf52840

.PHONY: flash-clue_nrf52840
flash-clue_nrf52840:
	LIBTOCK_PLATFORM=clue_nrf52840 cargo run --example $(EXAMPLE) $(features) \
		--target=thumbv7em-none-eabi $(release) -- --deploy=tockloader

.PHONY: clean
clean:
	cargo clean
	$(MAKE) -C tock clean
