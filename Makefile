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
	rustup install stable
	cargo +stable install elf2tab
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
	cargo +stable check --target-dir=target/stable --workspace \
		$(EXCLUDE_RUNTIME)
	LIBTOCK_PLATFORM=nrf52 cargo +stable check $(EXCLUDE_STD) \
		--target=thumbv7em-none-eabi --target-dir=target/stable --workspace

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

.PHONY: tab
tab:
	mkdir -p target/$(EXAMPLE)
	LINKER_FLASH=0x00040000 LINKER_RAM=0x20008000 cargo build --example $(EXAMPLE) $(features) --target=thumbv7em-none-eabi $(release)
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE) target/$(EXAMPLE)/cortex-m4.0x00040000.0x20008000.elf
	LINKER_FLASH=0x00042000 LINKER_RAM=0x2000a000 cargo build --example $(EXAMPLE) $(features) --target=thumbv7em-none-eabi $(release)
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE) target/$(EXAMPLE)/cortex-m4.0x00042000.0x2000a000.elf
	LINKER_FLASH=0x00048000 LINKER_RAM=0x20010000 cargo build --example $(EXAMPLE) $(features) --target=thumbv7em-none-eabi $(release)
	cp target/thumbv7em-none-eabi/release/examples/$(EXAMPLE) target/$(EXAMPLE)/cortex-m4.0x00048000.0x20010000.elf

	elf2tab --kernel-major 2 --kernel-minor 0 -n $(EXAMPLE) -o $(EXAMPLE).tab --stack 1024 \
		target/$(EXAMPLE)/cortex-m4.0x00040000.0x20008000.elf \
		target/$(EXAMPLE)/cortex-m4.0x00042000.0x2000a000.elf \
		target/$(EXAMPLE)/cortex-m4.0x00048000.0x20010000.elf \

# Creates the `make <BOARD> EXAMPLE=<EXAMPLE>` targets. Arguments:
#  1) The name of the platform to build for.
#
# A different --target-dir is passed for each platform to prevent race
# conditions between concurrent cargo run invocations. See
# https://github.com/tock/libtock-rs/issues/366 for more information.
define platform_build
.PHONY: $(1)
$(1):
	LIBTOCK_PLATFORM=$(1) cargo run --example $(EXAMPLE) $(features) \
		$(release) --target=thumbv7em-none-eabi --target-dir=target/$(1)
	mkdir -p target/tbf/$(1)
	cp target/$(1)/thumbv7em-none-eabi/release/examples/$(EXAMPLE).{tab,tbf} \
		target/tbf/$(1)
endef

# Creates the `make flash-<BOARD> EXAMPLE=<EXAMPLE>` targets. Arguments:
#  1) The name of the platform to flash for.
define platform_flash
.PHONY: flash-$(1)
flash-$(1):
	LIBTOCK_PLATFORM=$(1) cargo run --example $(EXAMPLE) $(features) \
		$(release) --target=thumbv7em-none-eabi --target-dir=target/$(1) -- \
		--deploy=tockloader
endef

$(eval $(call platform_build,apollo3))
$(eval $(call platform_build,esp32_c3_devkitm_1))
$(eval $(call platform_build,hail))
$(eval $(call platform_flash,hail))
$(eval $(call platform_build,imix))
$(eval $(call platform_flash,imix))
$(eval $(call platform_build,microbit_v2))
$(eval $(call platform_flash,microbit_v2))
$(eval $(call platform_build,nucleo_f429zi))
$(eval $(call platform_build,nucleo_f446re))
$(eval $(call platform_build,nrf52840))
$(eval $(call platform_flash,nrf52840))
$(eval $(call platform_build,raspberry_pi_pico))
$(eval $(call platform_build,nano_rp2040_connect))
$(eval $(call platform_build,stm32f3discovery))
$(eval $(call platform_build,stm32f412gdiscovery))
$(eval $(call platform_build,opentitan))
$(eval $(call platform_build,hifive1))
$(eval $(call platform_build,nrf52))
$(eval $(call platform_flash,nrf52))
$(eval $(call platform_build,imxrt1050))
$(eval $(call platform_build,msp432))
$(eval $(call platform_build,clue_nrf52840))
$(eval $(call platform_flash,clue_nrf52840))

.PHONY: clean
clean:
	cargo clean
	$(MAKE) -C tock clean
