# Make uses /bin/sh by default, which is a different shell on different OSes.
# Specify Bash instead so we don't have to test against a variety of shells.
SHELL := /usr/bin/env bash

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
artifact_dir=release
else
artifact_dir=debug
endif

# Rustup currently lacks the locking needed for concurrent use:
# https://github.com/rust-lang/rustup/issues/988. In particular, running
# concurrent cargo commands with a missing toolchain results in parallel rustup
# instances installing the same toolchain, corrupting that toolchain. To
# mitigate that issue, every target that uses the main (MSRV) toolchain should
# depend transitively on the `toolchain` target, so that the toolchain is
# installed before it is invoked concurrently. Note that we don't need to do
# this for the nightly toolchain because the nightly toolchain is only used by
# the `test` target, so this Makefile won't invoke it concurrently.
.PHONY: toolchain
toolchain:
	cargo -V

.PHONY: setup
setup: setup-qemu toolchain
	cargo install elf2tab

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
print-sizes: examples toolchain
	cargo run --release -p print_sizes

# Runs a libtock example in QEMU on a simulated HiFive board.
.PHONY: qemu-example
qemu-example: kernel-hifive toolchain
	LIBTOCK_PLATFORM="hifive1" cargo run --example "$(EXAMPLE)" -p libtock \
		--release --target=riscv32imac-unknown-none-elf -- --deploy qemu

# Build the examples on both a RISC-V target and an ARM target. We pick
# opentitan as the RISC-V target because it lacks atomics.
.PHONY: examples
examples: toolchain
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

# Arguments to pass to cargo to exclude demo crates.
EXCLUDE_RUNTIME := $(EXCLUDE_RUNTIME) --exclude st7789 --exclude st7789-slint

# Arguments to pass to cargo to exclude crates that cannot be tested by Miri. In
# addition to excluding libtock_runtime, Miri also cannot test proc macro crates
# (and in fact will generate broken data that causes cargo test to fail).
EXCLUDE_MIRI := $(EXCLUDE_RUNTIME) --exclude ufmt-macros

# Arguments to pass to cargo to exclude `std` and crates that depend on it. Used
# when we build a crate for an embedded target, as those targets lack `std`.
EXCLUDE_STD := --exclude libtock_unittest --exclude print_sizes \
               --exclude runner --exclude syscalls_tests \
               --exclude libtock_build_scripts

.PHONY: test
test: examples
	cargo test $(EXCLUDE_RUNTIME) --workspace
	LIBTOCK_PLATFORM=nrf52 cargo fmt --all -- --check
	cargo clippy --all-targets $(EXCLUDE_RUNTIME) --workspace
	LIBTOCK_PLATFORM=nrf52 cargo clippy $(EXCLUDE_STD) \
		--target=thumbv7em-none-eabi --workspace
	LIBTOCK_PLATFORM=hifive1 cargo clippy $(EXCLUDE_STD) \
		--target=riscv32imac-unknown-none-elf --workspace
	$(MAKE) apollo3-st7789
	$(MAKE) apollo3-st7789-slint
	cd nightly && \
		MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-symbolic-alignment-check" \
		cargo miri test $(EXCLUDE_MIRI) --manifest-path=../Cargo.toml \
		--target-dir=target --workspace
	echo '[ SUCCESS ] libtock-rs tests pass'

# Helper functions to define make targets to build for specific (flash, ram,
# target) compilation tuples.
#
# Inspiration from these answers:
# - https://stackoverflow.com/a/50357925
# - https://stackoverflow.com/a/9458230
#
# To create a compilation target for a specific architecture with specific flash
# and RAM addresses, use `fixed-target`:
#
# ```
# $(call fixed-target, F=0x00030000 R=0x20008000 T=thumbv7em-none-eabi A=cortex-m4)
# ```
#
# The "arguments" if you will are:
# - F = Flash Address: The address in flash the app is compiled for.
# - R = RAM Address: The address in RAM the app is compiled for.
# - T = Target: The cargo target to compile for.
# - A = Architecture: The Tock architecture name the target corresponds to.
#
# This technique uses two make variables internally to keep track of state:
# - `ELF_TARGETS`: This is the list of unique targets for each compilation
#   tuple. Each target invokes `cargo build` with the specified settings.
# - `ELF_LIST`: The is a list of .elf paths of the generated elfs (one per
#   compilation tuple). This is passed to `elf2tab` to generate the output .tab
#   file.
#
# Internally, what `fixed-target` does is define a new make target named the
# join of all of the F/R/T/A variables (with the `=` characters removed) and
# then assigns target variables to that new target to represent the compilation
# tuple values.
concat = $(subst =,,$(subst $(eval ) ,,$1))
fixed-target = $(foreach A,$1,$(eval $(call concat,$1): $A)) $(eval ELF_TARGETS += $(call concat,$1))

$(call fixed-target, F=0x00030000 R=0x20008000 T=thumbv7em-none-eabi A=cortex-m4)
$(call fixed-target, F=0x00038000 R=0x20010000 T=thumbv7em-none-eabi A=cortex-m4)

$(call fixed-target, F=0x00040000 R=0x10002000 T=thumbv7em-none-eabi A=cortex-m4)
$(call fixed-target, F=0x00048000 R=0x1000a000 T=thumbv7em-none-eabi A=cortex-m4)

$(call fixed-target, F=0x00040000 R=0x20008000 T=thumbv7em-none-eabi A=cortex-m4)
$(call fixed-target, F=0x00042000 R=0x2000a000 T=thumbv7em-none-eabi A=cortex-m4)
$(call fixed-target, F=0x00048000 R=0x20010000 T=thumbv7em-none-eabi A=cortex-m4)

$(call fixed-target, F=0x00080000 R=0x20006000 T=thumbv7em-none-eabi A=cortex-m4)
$(call fixed-target, F=0x00088000 R=0x2000e000 T=thumbv7em-none-eabi A=cortex-m4)

$(call fixed-target, F=0x403b0000 R=0x3fca2000 T=riscv32imc-unknown-none-elf A=riscv32imc)
$(call fixed-target, F=0x40440000 R=0x3fcaa000 T=riscv32imc-unknown-none-elf A=riscv32imc)

$(call fixed-target, F=0x10020000 R=0x20004000 T=thumbv6m-none-eabi A=cortex-m0)
$(call fixed-target, F=0x10028000 R=0x2000c000 T=thumbv6m-none-eabi A=cortex-m0)

$(ELF_TARGETS): toolchain
	LIBTOCK_LINKER_FLASH=$(F) LIBTOCK_LINKER_RAM=$(R) cargo build --example $(EXAMPLE) $(features) --target=$(T) $(release)
	@mkdir -p target/$(A).$(F).$(R)/
	@cp target/$(T)/$(artifact_dir)/examples/$(EXAMPLE) target/$(A).$(F).$(R)/
	$(eval ELF_LIST += target/$(A).$(F).$(R)/$(EXAMPLE),$(A).$(F).$(R))
# This target (`make tab`) is not parallel-safe
.PHONY: tab
tab: $(ELF_TARGETS)
	mkdir -p target/tab
	elf2tab --kernel-major 2 --kernel-minor 1 -n $(EXAMPLE) -o target/tab/$(EXAMPLE).tab --stack 1024 --minimum-footer-size 256 $(ELF_LIST)

# Creates the `make <BOARD> EXAMPLE=<EXAMPLE>` targets. Arguments:
#  1) The name of the platform to build for.
#  2) The target architecture the platform uses.
#
# A different --target-dir is passed for each platform to prevent race
# conditions between concurrent cargo run invocations. See
# https://github.com/tock/libtock-rs/issues/366 for more information.
define platform_build
.PHONY: $(1)
$(1): toolchain
	LIBTOCK_PLATFORM=$(1) cargo run --example $(EXAMPLE) $(features) \
		$(release) --target=$(2) --target-dir=target/$(1)
	mkdir -p target/tbf/$(1)
	cp target/$(1)/$(2)/release/examples/$(EXAMPLE).{tab,tbf} \
		target/tbf/$(1)

.PHONY: $(1)-st7789
$(1)-st7789: toolchain
	cd demos/st7789 && LIBTOCK_PLATFORM=$(1) cargo run $(features) \
		$(release) --target=$(2) --target-dir=target/$(1)
	mkdir -p target/tbf/$(1)
	cp demos/st7789/target/$(1)/$(2)/release/st7789.{tab,tbf} \
		target/tbf/$(1)

.PHONY: $(1)-st7789-slint
$(1)-st7789-slint: toolchain
	cd demos/st7789-slint && LIBTOCK_PLATFORM=$(1) cargo run $(features) \
		$(release) --target=$(2) --target-dir=target/$(1)
	mkdir -p target/tbf/$(1)
	cp demos/st7789-slint/target/$(1)/$(2)/release/st7789-slint.{tab,tbf} \
		target/tbf/$(1)
endef

# Creates the `make flash-<BOARD> EXAMPLE=<EXAMPLE>` targets. Arguments:
#  1) The name of the platform to flash for.
define platform_flash
.PHONY: flash-$(1)
flash-$(1): toolchain
	LIBTOCK_PLATFORM=$(1) cargo run --example $(EXAMPLE) $(features) \
		$(release) --target=$(2) --target-dir=target/flash-$(1) -- \
		--deploy=tockloader

.PHONY: flash-$(1)-st7789
flash-$(1)-st7789: toolchain
	cd demos/st7789 && LIBTOCK_PLATFORM=$(1) cargo run $(features) \
		$(release) --target=$(2) --target-dir=target/flash-$(1) -- \
		--deploy=tockloader

.PHONY: flash-$(1)-st7789-slint
flash-$(1)-st7789-slint: toolchain
	cd demos/st7789-slint && LIBTOCK_PLATFORM=$(1) cargo run $(features) \
		$(release) --target=$(2) --target-dir=target/flash-$(1) -- \
		--deploy=tockloader
endef

$(eval $(call platform_build,apollo3,thumbv7em-none-eabi))
$(eval $(call platform_build,esp32_c3_devkitm_1,riscv32imc-unknown-none-elf))
$(eval $(call platform_build,hail,thumbv7em-none-eabi))
$(eval $(call platform_flash,hail,thumbv7em-none-eabi))
$(eval $(call platform_build,imix,thumbv7em-none-eabi))
$(eval $(call platform_flash,imix,thumbv7em-none-eabi))
$(eval $(call platform_build,microbit_v2,thumbv7em-none-eabi))
$(eval $(call platform_flash,microbit_v2,thumbv7em-none-eabi))
$(eval $(call platform_build,nucleo_f429zi,thumbv7em-none-eabi))
$(eval $(call platform_build,nucleo_f446re,thumbv7em-none-eabi))
$(eval $(call platform_build,nrf52840,thumbv7em-none-eabi))
$(eval $(call platform_flash,nrf52840,thumbv7em-none-eabi))
$(eval $(call platform_build,raspberry_pi_pico,thumbv6m-none-eabi))
$(eval $(call platform_build,nano33ble,thumbv6m-none-eabi))
$(eval $(call platform_build,nano_rp2040_connect,thumbv6m-none-eabi))
$(eval $(call platform_build,stm32f3discovery,thumbv7em-none-eabi))
$(eval $(call platform_build,stm32f412gdiscovery,thumbv7em-none-eabi))
$(eval $(call platform_build,opentitan,riscv32imc-unknown-none-elf))
$(eval $(call platform_build,hifive1,riscv32imac-unknown-none-elf))
$(eval $(call platform_build,nrf52,thumbv7em-none-eabi))
$(eval $(call platform_flash,nrf52,thumbv7em-none-eabi))
$(eval $(call platform_build,imxrt1050,thumbv7em-none-eabi))
$(eval $(call platform_build,msp432,thumbv7em-none-eabi))
$(eval $(call platform_build,clue_nrf52840,thumbv7em-none-eabi))
$(eval $(call platform_flash,clue_nrf52840,thumbv7em-none-eabi))

# clean cannot safely be invoked concurrently with other actions, so we don't
# need to depend on toolchain. We also manually remove the nightly toolchain's
# target directory, in case the user doesn't want to install the nightly
# toolchain.
.PHONY: clean
clean:
	cargo clean
	rm -fr nightly/target/
	cd demos/st7789 && cargo clean
	$(MAKE) -C tock clean
