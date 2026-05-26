![Build Status](https://github.com/tock/libtock-rs/workflows/ci/badge.svg)

# libtock-rs

Rust userland library for Tock

Generally this library was tested with Tock [Release
2.1.1](https://github.com/tock/tock/releases/tag/release-2.1.1).

The library should work on all Tock boards, but currently apps must be compiled
for the flash and RAM address they are executed at. See [Fix
relocation](https://github.com/tock/libtock-rs/issues/28) for more details. You
may either compile a process binary especially for your board and use only a
single application written in rust at a time, or use the `make tab` target that
builds examples for a series of likely useful flash and RAM addresses.

## Getting Started

1.  Ensure you have [rustup](https://www.rustup.rs/) installed.

1.  Clone the repository:

    ```shell
    git clone --recursive https://github.com/tock/libtock-rs
    cd libtock-rs
    ```

1.  Install the dependencies:

    ```shell
    make setup
    ```

1.  Use `make` to build examples

    ```shell
    make nrf52 EXAMPLE=console # Builds the console example for the nrf52
    ```

## Using libtock-rs

The easiest way to start using libtock-rs is adding an example to the
`examples/` folder. We recommend starting by copying the `console` example, as
it is a simple example that shows you how to perform normal debug prints.

### Building for a specific board

To build your example for your board you can use

```shell
make <platform> EXAMPLE=<example>
```

An example can be flashed to your board after the build process by running:

```shell
make flash-<platform> EXAMPLE=<example>
```

This script does the following steps for you:

- cross-compile your program
- create a TAB (tock application bundle)
- if you have a J-Link compatible board connected: flash this TAB to your board (using tockloader)

### Enabling rust-embedded support

libtock-rs can be built to be compatible with the rust-embedded
[embedded_hal](https://docs.rs/embedded-hal/1.0.0/embedded_hal/index.html) by
including the following when running `make`

```shell
FEATURES=rust_embedded
```

If using libtock-rs or a sub-crate as a cargo dependency the `rust_embedded`
can also be enabled via Cargo.

### Building a generic TAB (Tock Application Bundle) file

To build your example for a variety of boards you can use

```shell
make tab EXAMPLE=<example>
```

To install the tab use tockloader

```shell
tockloader install target/tab/<example.tab>
```

Tockloader will determine which compiled version with the correct flash and RAM
addresses to use.

## Serial command protocol and host automation

If you expose a line-oriented command interface from a libtock-rs example,
keep the protocol contract stable so host scripts can run deterministically.

### Line endings

- Accept commands terminated with `\n` (LF) or `\r\n` (CRLF).
- Emit responses using `\n` (LF) only. This avoids host-dependent behavior and
  makes parsing predictable across Linux/macOS/Windows serial tooling.

### Timeout expectations

- Emit a startup banner as soon as the app is ready to accept commands.
- Host tools should use:
  - an initial, longer boot timeout (for banner detection), and
  - a shorter per-command timeout for normal request/response exchanges.
- Keep command handlers non-blocking where possible; if an operation is long,
  return an immediate status and report progress asynchronously.

### Success/error framing

- Return exactly one terminal status line for each command:
  - `OK <context>` for success
  - `ERR <code> <context>` for failure
- Avoid free-form status words such as `done`, `pass`, or `failed`; strict
  `OK`/`ERR` framing simplifies machine validation.

### Payload encoding (hex vs ASCII)

- Prefer ASCII tokens for control/config commands.
- For binary payloads, use explicit hex encoding (`0x`-prefixed bytes or
  contiguous hex text) and document which form is accepted.
- Do not mix inferred binary and text payloads on the same command without an
  explicit mode flag.

### Deterministic status output

- Print one status line per command in a fixed format.
- Do not interleave log/debug lines between a command and its terminal
  `OK`/`ERR` line unless logs are prefixed and documented as ignorable.
- Keep field order stable so host-side regexes remain reliable.

### Host-side `pyserial` example

The snippet below shows a small harness that waits for a banner, sends config
commands, starts GPIO/SPI/UART patterns, and validates `OK`/`ERR` outcomes.

```python
import re
import serial

PORT = "/dev/ttyACM0"
BAUD = 115200
BOOT_TIMEOUT_S = 8.0
CMD_TIMEOUT_S = 1.5

OK_RE = re.compile(r"^OK(?:\s+.*)?$")
ERR_RE = re.compile(r"^ERR(?:\s+\S+)?(?:\s+.*)?$")


def wait_for_banner(ser: serial.Serial, banner_prefix: str = "READY") -> str:
    """Read lines until a startup banner appears or timeout expires."""
    while True:
        line = ser.readline().decode("ascii", errors="replace").strip()
        if not line:
            raise TimeoutError(f"did not receive banner '{banner_prefix}'")
        if line.startswith(banner_prefix):
            return line


def send_cmd(ser: serial.Serial, cmd: str) -> str:
    ser.reset_input_buffer()
    ser.write((cmd + "\n").encode("ascii"))
    ser.flush()

    while True:
        line = ser.readline().decode("ascii", errors="replace").strip()
        if not line:
            raise TimeoutError(f"timeout waiting for status after: {cmd}")
        if OK_RE.match(line):
            return line
        if ERR_RE.match(line):
            raise RuntimeError(f"device rejected '{cmd}': {line}")
        # Optional log line; ignore and continue waiting for terminal status.


if __name__ == "__main__":
    with serial.Serial(PORT, BAUD, timeout=BOOT_TIMEOUT_S) as ser:
        banner = wait_for_banner(ser, banner_prefix="READY")
        print(f"banner: {banner}")

        # Switch to shorter command timeout after boot.
        ser.timeout = CMD_TIMEOUT_S

        # 1) Send config commands.
        send_cmd(ser, "CFG LINE_ENDING LF")
        send_cmd(ser, "CFG PAYLOAD HEX")
        send_cmd(ser, "CFG TIMEOUT_MS 250")

        # 2) Start peripheral test patterns.
        send_cmd(ser, "START GPIO PATTERN TOGGLE 20")
        send_cmd(ser, "START SPI PATTERN A5A5FF00")
        send_cmd(ser, "START UART PATTERN ASCII:HELLO")

        print("all commands completed with OK")
```


## License

libtock-rs is licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Submodules, as well as the code in the `ufmt` directory, have their own licenses.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

The contribution guidelines can be found here: [contribution guidelines](CONTRIBUTING.md)
