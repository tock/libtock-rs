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
	@echo " - opentitan"
	@echo " - hifive1"
	@echo " - nrf52"
	@echo " - apollo3"
	@echo
	@echo "Run 'make setup' to setup Rust to build libtock-rs."
	@echo "Run 'make <board>' to build libtock-rs for that board"
	@echo "    Set the DEBUG flag to enable the debug build"
	@echo "    Set the FEATURES flag to enable features"
	@echo "Run 'make flash-<board> EXAMPLE=<>' to flash EXAMPLE to that board"
	@echo "Run 'make test' to test any local changes you have made"

ifdef FEATURES
features=--features=$(FEATURES)
endif

ifndef DEBUG
release=--release
endif

.PHONY: setup
setup:
	rustup target add thumbv7em-none-eabi
	rustup target add riscv32imac-unknown-none-elf
	rustup target add riscv32imc-unknown-none-elf
	rustup component add rustfmt
	rustup component add clippy
	cargo install elf2tab --version 0.4.0
	cargo install stack-sizes

.PHONY: examples
examples:
	PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --examples
	PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --examples --features=alloc
	PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --example panic --features=custom_panic_handler,custom_alloc_error_handler
	PLATFORM=nrf52 cargo build --release --target=thumbv7em-none-eabi --example alloc_error --features=alloc,custom_alloc_error_handler
	PLATFORM=opentitan cargo build --release --target=riscv32imc-unknown-none-elf --examples # Important: This is testing a platform without atomics support

.PHONY: test
test:
	PLATFORM=nrf52 cargo fmt --all -- --check
	PLATFORM=nrf52 cargo clippy --workspace --all-targets
	PLATFORM=nrf52 cargo test --workspace
	make examples

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

.PHONY: nrf52840
nrf52840:
	PLATFORM=nrf52840 cargo build $(release) --target=thumbv7em-none-eabi --examples $(features)

.PHONY: flash-nrf52840
flash-nrf52840:
	PLATFORM=nrf52840 cargo run $(release) --target=thumbv7em-none-eabi --example $(EXAMPLE) $(features)

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

.PHONY: clean
clean:
	rm -rf target
	rm Cargo.lock
