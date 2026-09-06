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
	@printf " - %s\n" $(sort $(PLATFORMS))
	@echo
	@echo "Of those, the following can be flashed directly with 'make flash-<board>':"
	@printf " - %s\n" $(sort $(FLASH_PLATFORMS))
	@echo
	@echo "Run 'make setup' to setup Rust to build libtock-rs."
	@echo "Run 'make <board> EXAMPLE=<>' to build EXAMPLE for that board."
	@echo "Run 'make flash-<board> EXAMPLE=<>' to flash EXAMPLE to a tockloader-supported board."
	@echo
	@echo "These boards can be emulated, with no hardware, using 'make qemu-example-<board>':"
	@printf " - %s\n" $(sort $(QEMU_PLATFORMS))
	@echo
	@echo "Run 'make qemu-example-<board> EXAMPLE=<>' to run EXAMPLE in QEMU"
	@echo "Run 'make qemu-examples EXAMPLE=<>' to run EXAMPLE on every one of them"
	@echo "Run 'make clean-tock-cache' to discard the Tock checkout those use"
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
setup: toolchain
	cargo install elf2tab

# Prints out the sizes of the example binaries.
.PHONY: print-sizes
print-sizes: examples toolchain
	cargo run --release -p print_sizes

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
	--exclude libtock_debug_panic --exclude libtock_small_panic --exclude embedded_graphics_libtock

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
	cd nightly && \
		MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-symbolic-alignment-check" \
		cargo miri test $(EXCLUDE_MIRI) --manifest-path=../Cargo.toml \
		--target-dir=target --workspace
	echo '[ SUCCESS ] libtock-rs tests pass'

include Targets.mk
include Makefile.qemu

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
# PLATFORMS accumulates the canonical list to prevent drift.
#
# A different --target-dir is passed for each platform to prevent race
# conditions between concurrent cargo run invocations. See
# https://github.com/tock/libtock-rs/issues/366 for more information.
define platform_build
PLATFORMS += $(1)
.PHONY: $(1)
$(1): toolchain
	LIBTOCK_PLATFORM=$(1) cargo run --example $(EXAMPLE) $(features) \
		$(release) --target=$(2) --target-dir=target/$(1)
	mkdir -p target/tbf/$(1)
	cp target/$(1)/$(2)/release/examples/$(EXAMPLE).{tab,tbf} \
		target/tbf/$(1)
endef

# Creates the `make flash-<BOARD> EXAMPLE=<EXAMPLE>` targets. Arguments:
#  1) The name of the platform to flash for.
#  2) The target architecture the platform uses.
#
# FLASH_PLATFORMS accumulates the canonical list to prevent drift.
define platform_flash
FLASH_PLATFORMS += $(1)
.PHONY: flash-$(1)
flash-$(1): toolchain
	LIBTOCK_PLATFORM=$(1) cargo run --example $(EXAMPLE) $(features) \
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
$(eval $(call platform_build,pico_explorer_base,thumbv6m-none-eabi))
$(eval $(call platform_build,nano33ble,thumbv6m-none-eabi))
$(eval $(call platform_build,nano_rp2040_connect,thumbv6m-none-eabi))
$(eval $(call platform_build,qemu_rv32_virt,riscv32imac-unknown-none-elf))
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
$(eval $(call platform_build,psc3m5_evk,thumbv8m.main-none-eabi))

# The demo apps. Each is a standalone cargo workspace with its own Makefile and
# its own target directory, so blanket rules have to visit each of them.
DEMOS := demos/embedded_graphics/spin \
         demos/embedded_graphics/buttons \
         demos/st7789 \
         demos/st7789-slint

.PHONY: demos
demos:
	@for demo in $(DEMOS); do \
		echo "$(MAKE) -C $$demo"; \
		$(MAKE) -C "$$demo" || exit 1; \
	done

# clean cannot safely be invoked concurrently with other actions, so we don't
# need to depend on toolchain. We also manually remove the nightly toolchain's
# target directory, in case the user doesn't want to install the nightly
# toolchain.
.PHONY: clean
clean: clean-qemu
	cargo clean
	rm -fr nightly/target/
	@for demo in $(DEMOS); do (cd "$$demo" && cargo clean) || exit 1; done
