# Signal Generator CLI

`signal_generator_cli` is a simple ASCII command-line interface for exercising GPIO, SPI, and UART-related command handling from a libtock-rs userspace app.

The CLI prints responses in this format:

- Success: `OK <command> <fields...>`
- Failure: `ERR <command> <error_code> <message>`

## Build and run

Build the example:

```sh
cargo build --example signal_generator_cli
```

Run it with your usual board workflow. Once connected, type commands as ASCII lines terminated with `Enter`.

## Current behavior notes

- The console interface uses the Tock console driver, so command input and output go over the board console channel.
- GPIO commands actively drive GPIO pins.
- SPI commands actively call the SPI controller driver.
- UART commands currently parse, validate, store, and report UART state, but this example does not currently call a UART transmit syscall. The `uart tx` and `uart repeat` commands report the requested payload and settings, but do not yet send bytes to hardware.
- In the checked-in example source, `UART_SECONDARY_SUPPORTED` is currently hardcoded to `true`.

## Quick start

Typical session:

```text
help
caps
setfreq 2000
gpio map 0 2
gpio mode 0 square
gpio timing 0 1000 500
gpio start 0
status
gpio status 0
gpio stop 0
```

## Top-level commands

### `help`

Shows the supported command groups.

Example:

```text
help
```

### `caps`

Shows feature and buffer limits.

Example:

```text
caps
```

### `reset`

Stops active generators and restores default state.

Example:

```text
reset
```

### `start`

Sets the top-level `running` flag.

Example:

```text
start
```

### `stop`

Clears the top-level `running` flag and stops active outputs.

Example:

```text
stop
```

### `status`

Shows the top-level status.

Example:

```text
status
```

### `setfreq <frequency_hz>`

Sets the top-level frequency field.

Example:

```text
setfreq 1000
setfreq 0x7D0
```

## Number formats

Many commands accept either decimal or hexadecimal values.

- Decimal example: `1000`
- Hex example: `0x3E8`

Hex byte payloads must have an even number of hex digits.

Valid examples:

```text
0xAA55
0x01020304
```

Invalid example:

```text
0xABC
```

## GPIO commands

GPIO has 8 logical channels: `0` through `7`.

### `gpio map <channel> <pin>`

Associates a logical channel with a physical GPIO pin.

Examples:

```text
gpio map 0 2
gpio map 1 3
```

### `gpio mode <channel> <mode>`

Supported modes:

- `off`
- `high`
- `low`
- `square`
- `burst`
- `pattern`

Examples:

```text
gpio mode 0 high
gpio mode 0 low
gpio mode 0 square
gpio mode 0 burst
gpio mode 0 pattern
```

### `gpio timing <channel> <period_us> [high_us]`

Sets square-wave or burst timing.

- If `high_us` is omitted, it defaults to half the period.
- `high_us` must not exceed `period_us`.
- The allowed interval range is `100` to `60000000` microseconds.

Examples:

```text
gpio timing 0 1000 500
gpio timing 0 2000
gpio timing 1 0x3E8 0x1F4
```

### `gpio pattern <channel> <hex_bits> <step_us>`

Configures a repeating bit pattern.

Examples:

```text
gpio pattern 0 0xA5 500
gpio pattern 1 0xF0F0 1000
```

### `gpio start <channel|all>`

Starts one channel or all channels.

Examples:

```text
gpio start 0
gpio start all
```

### `gpio stop <channel|all>`

Stops one channel or all channels.

Examples:

```text
gpio stop 0
gpio stop all
```

### `gpio status [channel]`

Shows channel status for one channel or all channels.

Examples:

```text
gpio status
gpio status 0
```

### Common GPIO workflows

Set GPIO2 high:

```text
gpio map 0 2
gpio mode 0 high
gpio start 0
```

Generate a 1 kHz square wave with 50% duty cycle on GPIO3:

```text
gpio map 1 3
gpio mode 1 square
gpio timing 1 1000 500
gpio start 1
```

Generate a two-pulse burst on GPIO4:

```text
gpio map 2 4
gpio mode 2 burst
gpio timing 2 1000 250
gpio start 2
```

Generate a pattern on GPIO5:

```text
gpio map 3 5
gpio mode 3 pattern
gpio pattern 3 0xA 1000
gpio start 3
```

## SPI commands

SPI commands use the Tock SPI controller driver and actively talk to hardware.

### `spi cfg <hz> <mode> <cs>`

Sets SPI bus speed, mode, and chip-select index.

Supported modes:

- `mode0`
- `mode1`
- `mode2`
- `mode3`

Examples:

```text
spi cfg 1000000 mode0 0
spi cfg 4000000 mode3 1
spi cfg 0xF4240 mode1 0
```

### `spi tx <hex_bytes>`

Writes bytes over SPI.

Examples:

```text
spi tx 0x9F
spi tx 0xAABBCCDD
spi tx 0x0102030405060708
```

### `spi txrx <hex_bytes> <rx_len>`

Writes bytes, then reads back `rx_len` bytes.

Examples:

```text
spi txrx 0x9F 3
spi txrx 0x0B000000 8
spi txrx 0xA5 0x04
```

### `spi repeat <hex_bytes> <count> <interval_us>`

Repeatedly transmits the same payload with a delay between sends.

Examples:

```text
spi repeat 0x55AA 10 1000
spi repeat 0xDEADBEEF 3 500000
spi repeat 0x01 0x0A 0x3E8
```

### `spi stop`

Stops an active SPI repeat sequence.

Example:

```text
spi stop
```

### `spi status`

Shows current SPI settings and last TX/RX buffers.

Example:

```text
spi status
```

### Common SPI workflows

Read a JEDEC ID from a flash device:

```text
spi cfg 1000000 mode0 0
spi txrx 0x9F 3
```

Write a simple command byte repeatedly once per millisecond:

```text
spi cfg 2000000 mode0 0
spi repeat 0x55 100 1000
```

Write a command and payload in one transfer:

```text
spi cfg 4000000 mode0 0
spi tx 0x02000000AABBCCDD
```

## UART commands

UART commands are enabled by the example's secondary-UART capability gate.

Important: in the current implementation, these commands manage and report UART state, but they do not actually transmit bytes to hardware yet.

### `uart cfg <port> <baud> <data_bits> <parity> <stop_bits>`

Configures logical UART state.

- `port` must be non-zero
- `data_bits` must be `5` through `8`
- `parity` must be `none`, `even`, or `odd`
- `stop_bits` must be `1` or `2`

Examples:

```text
uart cfg 1 115200 8 none 1
uart cfg 1 9600 7 even 1
uart cfg 1 57600 8 odd 2
```

### `uart tx <port> <hex|ascii> <payload...>`

Stores a UART payload and reports what would be transmitted.

Hex payload examples:

```text
uart tx 1 hex 0x55
uart tx 1 hex 0x48656C6C6F0D0A
uart tx 1 hex 0xAABBCCDDEEFF
```

ASCII payload examples:

```text
uart tx 1 ascii hello
uart tx 1 ascii hello world
uart tx 1 ascii AT+GMR
```

### `uart repeat <port> <payload> <count|infinite> <interval_us>`

Stores a repeating UART payload plan and reports the configured repeat state.

- If the payload starts with `0x` or `0X`, it is treated as hex.
- Otherwise it is treated as ASCII.
- `count` may be a positive integer or `infinite`.

Hex payload examples:

```text
uart repeat 1 0x55AA 10 1000
uart repeat 1 0x0D0A infinite 500000
```

ASCII payload examples:

```text
uart repeat 1 ping 5 1000000
uart repeat 1 hello infinite 2000000
```

### `uart stop <port>`

Stops the stored UART repeat state.

Example:

```text
uart stop 1
```

### `uart status [port]`

Shows the stored UART configuration and last payload.

Examples:

```text
uart status
uart status 1
```

### Common UART workflows

Configure a typical 115200 8N1 UART state:

```text
uart cfg 1 115200 8 none 1
uart status 1
```

Prepare a one-shot ASCII payload:

```text
uart tx 1 ascii Hello UART
uart status 1
```

Prepare a hex payload:

```text
uart tx 1 hex 0x01020304
uart status 1
```

Prepare a repeated message plan:

```text
uart repeat 1 ping 10 500000
uart status 1
uart stop 1
```

## I2S command

I2S is currently reported as unsupported.

Example:

```text
i2s
```

Expected result:

```text
ERR i2s ERR_UNSUPPORTED i2s_not_available
```

## Limits and validation

- Maximum input line length: `96` bytes
- SPI TX and RX buffer size: `64` bytes
- UART payload buffer size: `64` bytes
- Minimum repeat interval: `100` microseconds
- Maximum repeat interval: `60000000` microseconds
- Maximum simultaneously active generators: `4`

## Troubleshooting

### `ERR_PARSE`

The command syntax is wrong or a numeric field could not be parsed.

Common causes:

- Missing arguments
- Too many arguments
- Invalid decimal or hex input
- Invalid enum-like token such as an unsupported mode or parity

### `ERR_RANGE`

The command syntax is correct, but a value is outside the allowed range.

Common causes:

- `setfreq 0`
- `gpio timing` interval too small or too large
- `high_us` larger than `period_us`
- Empty SPI or UART payload
- Payload longer than the fixed buffer
- `count` set to `0`

### `ERR_DRIVER`

The underlying driver call failed.

This currently applies to GPIO and SPI operations. The current UART command implementation does not yet invoke a UART driver.