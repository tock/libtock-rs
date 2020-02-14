# Available flash commands

- `cargo rriscv32iamc`/`cargo rrv32iamc`: Use the `riscv32iamc-unknown-none-elf` target
- `cargo rriscv32imc`/`cargo rrv32imc`: Use the `riscv32imc-unknown-none-elf` target
- `cargo rthumbv7em`/`cargo rtv7em`: Use the `thumbv7em-none-eabi` target

Before flashing, write your board name to the environment variable `PLATFORM` or to a file named `platform`