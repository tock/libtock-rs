# Helper makefile to define build location targets for many common tock
# platforms.
#
# To use:
#
#     include Targets.mk

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

$(call fixed-target, F=0x403b0000 R=0x3fca2000 T=riscv32imc-unknown-none-elf A=rv32imc)
$(call fixed-target, F=0x40440000 R=0x3fcaa000 T=riscv32imc-unknown-none-elf A=rv32imc)

$(call fixed-target, F=0x80100000 R=0x80300000 T=riscv32imac-unknown-none-elf A=rv32imac)
$(call fixed-target, F=0x80110000 R=0x80310000 T=riscv32imac-unknown-none-elf A=rv32imac)
$(call fixed-target, F=0x80130000 R=0x80330000 T=riscv32imac-unknown-none-elf A=rv32imac)
$(call fixed-target, F=0x80180000 R=0x80380000 T=riscv32imac-unknown-none-elf A=rv32imac)

$(call fixed-target, F=0x10020000 R=0x20004000 T=thumbv6m-none-eabi A=cortex-m0)
$(call fixed-target, F=0x10028000 R=0x2000c000 T=thumbv6m-none-eabi A=cortex-m0)

$(call fixed-target, F=0x10040000 R=0x20020000 T=thumbv8m.main-none-eabi A=cortex-m33)
$(call fixed-target, F=0x10060000 R=0x20028000 T=thumbv8m.main-none-eabi A=cortex-m33)