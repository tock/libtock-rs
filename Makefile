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
	@echo " - nucleo_f429zi"
	@echo " - nucleo_f446re"
	@echo " - opentitan"
	@echo " - hifive1"
	@echo " - nrf52"
	@echo " - imxrt1050"
	@echo " - apollo3"
	@echo " - stm32f3discovery"
	@echo
	@echo "Run 'make setup' to setup Rust to build libtock-rs."
	@echo "Run 'make <board>' to build libtock-rs for that board"
	@echo "    Set the DEBUG flag to enable the debug build"
	@echo "    Set the FEATURES flag to enable features"
	@echo "Run 'make flash-<board> EXAMPLE=<>' to flash EXAMPLE to that board"
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
	cargo install elf2tab --version 0.6.0
	cargo install stack-sizes
	cargo miri setup

# Sets up QEMU in the tock/ directory. We use Tock's QEMU which may contain
# patches to better support boards that Tock supports.
.PHONY: setup-qemu
setup-qemu:
	CI=true $(MAKE) -C tock ci-setup-qemu

# Builds a Tock kernel for the HiFive board for use by QEMU tests.
.PHONY: kernel-hifive
kernel-hifive:
	$(MAKE) -C tock/boards/hifive1 \
		$(CURDIR)/tock/target/riscv32imac-unknown-none-elf/release/hifive1.elf

# Builds a Tock 2.0 kernel for the HiFive board for use by QEMU tests.
# TODO: After Tock 2.0 is released, we should merge the tock/ and tock2/
# submodules and only build Tock 2.0 kernels.
.PHONY: kernel-hifive-2
kernel-hifive-2:
	$(MAKE) -C tock2/boards/hifive1 \
		$(CURDIR)/tock2/target/riscv32imac-unknown-none-elf/release/hifive1.elf

# Prints out the sizes of the example binaries.
.PHONY: print-sizes
print-sizes: examples
	cargo run --release -p print_sizes

# Runs the libtock_test tests in QEMU on a simulated HiFive board.
.PHONY: test-qemu-hifive
test-qemu-hifive: kernel-hifive
	PLATFORM=hifive1 cargo rrv32imac --example libtock_test --features=alloc \
		--features=__internal_disable_gpio_in_integration_test \
		--features=__internal_disable_timer_in_integration_test
	cargo run -p test_runner

.PHONY: examples
examples:
	PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --examples -p libtock -p libtock_core
	PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --examples --features=alloc
	PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --example panic --features=custom_panic_handler,custom_alloc_error_handler
	PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --example alloc_error --features=alloc,custom_alloc_error_handler
	# Important: This tests a platform without atomic instructions.
	PLATFORM=opentitan cargo build --release --target=riscv32imc-unknown-none-elf --examples -p libtock -p libtock_core

# Arguments to pass to cargo to exclude libtock_runtime and crates that depend
# on libtock_runtime. Used when we need to build a crate for the host OS, as
# libtock_runtime only supports running on Tock.
EXCLUDE_RUNTIME := --exclude libtock2 --exclude libtock_runtime

# Arguments to pass to cargo to exclude `std` and crates that depend on it. Used
# when we build a crate for an embedded target, as those targets lack `std`.
EXCLUDE_STD := --exclude libtock_unittest --exclude print_sizes \
               --exclude syscalls_tests --exclude test_runner

.PHONY: test
test: examples test-qemu-hifive
	# TODO: When we have a working embedded test harness, change the libtock2
	# builds to --all-targets rather than --examples.
	# Build libtock2 on both a RISC-V target and an ARM target. We pick
	# opentitan as the RISC-V target because it lacks atomics.
	LIBTOCK_PLATFORM=opentitan cargo build --examples --release \
		--target=riscv32imc-unknown-none-elf -p libtock2
	LIBTOCK_PLATFORM=nrf52 cargo build --examples --release \
		--target=thumbv7em-none-eabi -p libtock2
	LIBTOCK_PLATFORM=nrf52 PLATFORM=nrf52 cargo fmt --all -- --check
	PLATFORM=nrf52 cargo clippy --all-targets $(EXCLUDE_RUNTIME) --workspace
	# TODO: Add a clippy invocation for an ARM platform, once the Tock 1.0
	# crates are removed. It is omitted because the Tock 1.0 crates don't
	# currently pass clippy on ARM.
	LIBTOCK_PLATFORM=hifive1 PLATFORM=hifive1 cargo clippy $(EXCLUDE_STD) \
		--target=riscv32imac-unknown-none-elf --workspace
	PLATFORM=nrf52 cargo miri test $(EXCLUDE_RUNTIME) --workspace
	MIRIFLAGS="-Zmiri-symbolic-alignment-check -Zmiri-track-raw-pointers" \
		PLATFORM=nrf52 cargo miri test $(EXCLUDE_RUNTIME) --workspace
	echo '[ SUCCESS ] libtock-rs tests pass'

.PHONY: analyse-stack-sizes
analyse-stack-sizes:
	cargo stack-sizes $(release) --example $(EXAMPLE) $(features) -- -Z emit-stack-sizes

.PHONY: apollo3
apollo3:
	PLATFORM=apollo3 cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)

.PHONY: flash-apollo3
flash-apollo3:
	PLATFORM=apollo3 cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

.PHONY: hail
hail:
	PLATFORM=hail cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)

.PHONY: flash-hail
flash-hail:
	PLATFORM=hail cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

.PHONY: nucleo_f429zi
nucleo_f429zi:
	PLATFORM=nucleo_f429zi cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)

.PHONY: flash-nucleo_f429zi
flash-nucleo_f429zi:
	PLATFORM=nucleo_f429zi cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

.PHONY: nucleo_f446re
nucleo_f446re:
	PLATFORM=nucleo_f446re cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)

.PHONY: flash-nucleo_f446re
flash-nucleo_f446re:
	PLATFORM=nucleo_f446re cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

.PHONY: nrf52840
nrf52840:
	PLATFORM=nrf52840 cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)

.PHONY: flash-nrf52840
flash-nrf52840:
	PLATFORM=nrf52840 cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

.PHONY: stm32f3discovery
stm32f3discovery:
	PLATFORM=stm32f3discovery cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)
	
.PHONY: flash-stm32f3discovery
flash-stm32f3discovery:
	PLATFORM=stm32f3discovery cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

.PHONY: opentitan
opentitan:
	PLATFORM=opentitan cargo build $(release) --target=riscv32imc-unknown-none-elf --examples $(features)

.PHONY: flash-opentitan
flash-opentitan:
	PLATFORM=opentitan cargo run $(release) --target=riscv32imc-unknown-none-elf --example $(EXAMPLE) $(features)

.PHONY: hifive1
hifive1:
	PLATFORM=hifive1 cargo build $(release) --target=riscv32imac-unknown-none-elf --examples $(features)

.PHONY: flash-hifive1
flash-hifive1:
	PLATFORM=hifive1 cargo run $(release) --target=riscv32imac-unknown-none-elf --example $(EXAMPLE) $(features)

.PHONY: nrf52
nrf52:
	PLATFORM=nrf52 cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)

.PHONY: flash-nrf52
flash-nrf52:
	PLATFORM=nrf52 cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

.PHONY: imxrt1050
imxrt1050:
	PLATFORM=imxrt1050 cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)

.PHONY: flash-imxrt1050
flash-imxrt1050:
	PLATFORM=imxrt1050 cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

.PHONY: msp432
msp432:
	PLATFORM=msp432 cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)

.PHONY: flash-msp432
flash-msp432:
	PLATFORM=msp432 cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

.PHONY: clean
clean:
	cargo clean
	$(MAKE) -C tock clean
