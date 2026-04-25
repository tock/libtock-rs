//! A simple ASCII command-line interface for a toy signal generator.
//!
//! Response format:
//! - Success: `OK <command> <fields...>`
//! - Failure: `ERR <command> <error_code> <message>`

#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x600}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_LINE_LEN: usize = 96;

struct SignalState {
    running: bool,
    frequency_hz: u32,
}

impl SignalState {
    fn new() -> Self {
        Self {
            running: false,
            frequency_hz: 1_000,
        }
    }

    fn reset(&mut self) {
        self.running = false;
        self.frequency_hz = 1_000;
    }
}

fn print_boot_banner() {
    writeln!(
        Console::writer(),
        "signal-generator-cli v{VERSION} caps=help,caps,reset,start,stop,setfreq,status radix=dec,0x max_line={MAX_LINE_LEN}"
    )
    .unwrap();
}

fn emit_ok(command: &str, fields: &str) {
    if fields.is_empty() {
        writeln!(Console::writer(), "OK {command}").unwrap();
    } else {
        writeln!(Console::writer(), "OK {command} {fields}").unwrap();
    }
}

fn emit_err(command: &str, code: &str, message: &str) {
    writeln!(Console::writer(), "ERR {command} {code} {message}").unwrap();
}

fn parse_u32_ascii(token: &str) -> Result<u32, &'static str> {
    if token.is_empty() {
        return Err("empty_number");
    }

    if let Some(hex) = token.strip_prefix("0x") {
        return u32::from_str_radix(hex, 16).map_err(|_| "invalid_hex");
    }

    if let Some(hex) = token.strip_prefix("0X") {
        return u32::from_str_radix(hex, 16).map_err(|_| "invalid_hex");
    }

    token.parse::<u32>().map_err(|_| "invalid_decimal")
}

fn execute_command(line: &str, state: &mut SignalState) {
    let mut parts = line.split_whitespace();
    let Some(command) = parts.next() else {
        emit_err("input", "EMPTY", "empty_command");
        return;
    };

    match command {
        "help" => {
            if parts.next().is_some() {
                emit_err("help", "ARG", "unexpected_arguments");
                return;
            }
            emit_ok(
                "help",
                "commands=help,caps,reset,start,stop,setfreq,status setfreq=<u32(dec|0xHEX)>",
            );
        }
        "caps" => {
            if parts.next().is_some() {
                emit_err("caps", "ARG", "unexpected_arguments");
                return;
            }
            emit_ok(
                "caps",
                "features=help,caps,reset,start,stop,setfreq,status ascii=1 radix=dec,0x max_line=96",
            );
        }
        "reset" => {
            if parts.next().is_some() {
                emit_err("reset", "ARG", "unexpected_arguments");
                return;
            }
            state.reset();
            emit_ok("reset", "running=0 frequency_hz=1000");
        }
        "start" => {
            if parts.next().is_some() {
                emit_err("start", "ARG", "unexpected_arguments");
                return;
            }
            state.running = true;
            emit_ok("start", "running=1");
        }
        "stop" => {
            if parts.next().is_some() {
                emit_err("stop", "ARG", "unexpected_arguments");
                return;
            }
            state.running = false;
            emit_ok("stop", "running=0");
        }
        "status" => {
            if parts.next().is_some() {
                emit_err("status", "ARG", "unexpected_arguments");
                return;
            }
            writeln!(
                Console::writer(),
                "OK status running={} frequency_hz={}",
                if state.running { 1 } else { 0 },
                state.frequency_hz
            )
            .unwrap();
        }
        "setfreq" => {
            let Some(token) = parts.next() else {
                emit_err("setfreq", "ARG", "missing_frequency");
                return;
            };

            if parts.next().is_some() {
                emit_err("setfreq", "ARG", "too_many_arguments");
                return;
            }

            let frequency_hz = match parse_u32_ascii(token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("setfreq", "ARG", msg);
                    return;
                }
            };

            if frequency_hz == 0 {
                emit_err("setfreq", "RANGE", "frequency_must_be_nonzero");
                return;
            }

            state.frequency_hz = frequency_hz;
            writeln!(Console::writer(), "OK setfreq frequency_hz={frequency_hz}").unwrap();
        }
        _ => emit_err(command, "UNKNOWN", "unknown_command"),
    }
}

fn main() {
    print_boot_banner();
    let mut state = SignalState::new();

    let mut line_buf = [0u8; MAX_LINE_LEN + 1];
    let mut line_len = 0usize;
    let mut single_byte_buf = [0u8; 1];

    loop {
        let (_, err) = Console::read(&mut single_byte_buf);
        if err.is_err() {
            emit_err("input", "IO", "read_failed");
            continue;
        }

        let byte = single_byte_buf[0];

        if byte > 0x7F {
            emit_err("input", "ASCII", "non_ascii_input");
            line_len = 0;
            continue;
        }

        if byte == b'\n' || byte == b'\r' {
            if line_len == 0 {
                continue;
            }

            let line = core::str::from_utf8(&line_buf[..line_len]).unwrap_or_default();
            execute_command(line, &mut state);
            line_len = 0;
            continue;
        }

        if byte.is_ascii_control() {
            emit_err("input", "ASCII", "unsupported_control_character");
            line_len = 0;
            continue;
        }

        if line_len >= MAX_LINE_LEN {
            emit_err("input", "TOO_LONG", "line_too_long");
            line_len = 0;
            continue;
        }

        line_buf[line_len] = byte;
        line_len += 1;
    }
}
