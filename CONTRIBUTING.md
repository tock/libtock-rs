# PR Workflow

We use the bors-ng bot to merge PRs. In short, when someone replies `bors r+`,
your PR has been scheduled for final tests and will be automatically merged. If
a maintainer replies `bors delegate+`, then you have been granted the authority
to merge your own PR (usually this will happen if there are some trivial
changes required). For a full list of bors commands,
[see the bors documentation](https://bors.tech/documentation/).

# Tests

Our aim is to provide a number of tests to be safe from regression. Currently,
the unit tests and the linting is automated, however, the integration tests
have to be run manually.

## Compilation

`libtock-rs` currently has the following compilation targets

- `riscv32imac-unknown-none-elf`
- `riscv32imc-unknown-none-elf`
- `thumbv7em-none-eabi`

You can trigger a test build of the library and the examples using the script `build_examples.sh`.

## Unit Testing and Linting

There a a number of tests which run in our travis-ci environment. You can run them
using `cargo test --workspace`.

## Integration tests

If you have an nRF52 DK you can run the integration tests as follows.
The pins P0.03 and P0.04 need to be connected (on an nRF52 DK). Then do the following:

- connect your device to your computer
- open a console, e.g. `tockloader listen`
- run the tests: `PLATFORM=nrf52 cargo rtv7em hardware_test`

The expected output on the UART console will be as follows.

```
[test-results]
heap_test = "Heap works."
formatting =  works
should_be_one = 1
gpio_works = true
trait_obj_value_usize = 1
trait_obj_value_string = string
callbacks_work = true
all_tests_run = true
```
