# Makefile for the demo app.

# Crate name of the demo app
DEMO := buttons

all: tab

include ../../../Targets.mk

$(ELF_TARGETS):
	LIBTOCK_LINKER_FLASH=$(F) LIBTOCK_LINKER_RAM=$(R) cargo build --target=$(T) --release
	@mkdir -p target/$(A).$(F).$(R)/
	@cp target/$(T)/release/$(DEMO) target/$(A).$(F).$(R)/
	$(eval ELF_LIST += target/$(A).$(F).$(R)/$(DEMO),$(A).$(F).$(R))

# This target (`make tab`) is not parallel-safe
.PHONY: tab
tab: $(ELF_TARGETS)
	@mkdir -p target/tab
	elf2tab --kernel-major 2 --kernel-minor 1 -n $(DEMO) -o target/tab/$(DEMO).tab --stack 1024 --minimum-footer-size 256 $(ELF_LIST)
