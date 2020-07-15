# Tests

Our aim is to provide a number of tests to be safe from regression. Currently,
the unit tests and the linting is automated, however, the integration tests
have to be run manually.

## Compilation

`libtock-rs` currently has the following compilation targets

- `riscv32imac-unknown-none-elf`
- `riscv32imc-unknown-none-elf`
- `thumbv7em-none-eabi`

You can trigger a test build of the library and the examples using `make test`.

## Unit Testing and Linting

There a a number of tests which run in our travis-ci environment. You can run them
using `make test`.

## Integration tests

If you have an nRF52 DK you can run the integration tests as follows.
The pins P0.03 and P0.04 need to be connected (on an nRF52 DK). Then do the following:

- connect your device to your computer
- open a console, e.g. `tockloader listen`
- run the tests: `make flash-nrf52 EXAMPLE=libtock_test FEATURES=alloc`

The expected output on the UART console will be as follows.

```
[      OK ] Console
[      OK ] static mut
[      OK ] Dynamic dispatch
[      OK ] Formatting
[      OK ] Heap
[      OK ] Drivers only instantiable once
[      OK ] Callbacks
[      OK ] GPIO initialization
[      OK ] GPIO activation
[      OK ] GPIO read/write
[      OK ] Test suite finished with state SUCCESS
```

# PR Review Workflow

Our code review practices are documented in our [Code Review](doc/CodeReview.md)
document.
