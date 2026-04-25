//! A simple ASCII command-line interface for a toy signal generator.
//!
//! Response format:
//! - Success: `OK <command> <fields...>`
//! - Failure: `ERR <command> <error_code> <message>`

#![no_main]
#![no_std]

use core::fmt::Write;
use libtock::alarm::Alarm;
use libtock::console::Console;
use libtock::gpio::Gpio;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x600}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_LINE_LEN: usize = 96;
const GPIO_CHANNEL_COUNT: usize = 8;

struct SignalState {
    running: bool,
    frequency_hz: u32,
    gpio: GpioEngine,
}

impl SignalState {
    fn new() -> Self {
        Self {
            running: false,
            frequency_hz: 1_000,
            gpio: GpioEngine::new(),
        }
    }

    fn reset(&mut self) {
        self.running = false;
        self.frequency_hz = 1_000;
        self.gpio.reset();
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ChannelMode {
    Off,
    High,
    Low,
    Square,
    Burst,
    Pattern,
}

impl ChannelMode {
    fn as_str(&self) -> &'static str {
        match self {
            ChannelMode::Off => "off",
            ChannelMode::High => "high",
            ChannelMode::Low => "low",
            ChannelMode::Square => "square",
            ChannelMode::Burst => "burst",
            ChannelMode::Pattern => "pattern",
        }
    }

    fn parse(token: &str) -> Option<Self> {
        match token {
            "off" => Some(ChannelMode::Off),
            "high" => Some(ChannelMode::High),
            "low" => Some(ChannelMode::Low),
            "square" => Some(ChannelMode::Square),
            "burst" => Some(ChannelMode::Burst),
            "pattern" => Some(ChannelMode::Pattern),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
struct ChannelConfig {
    pin_index: Option<u32>,
    mode: ChannelMode,
    period_us: u32,
    high_us: u32,
    repetitions: u32,
    pattern_bits: Option<u64>,
    pattern_len: u8,
    pattern_step_us: u32,
}

impl ChannelConfig {
    const fn new() -> Self {
        Self {
            pin_index: None,
            mode: ChannelMode::Off,
            period_us: 1_000,
            high_us: 500,
            repetitions: 1,
            pattern_bits: None,
            pattern_len: 0,
            pattern_step_us: 0,
        }
    }
}

#[derive(Copy, Clone)]
struct ChannelRuntime {
    active: bool,
    start_us: u64,
    last_level: Option<bool>,
    transitions: u32,
}

impl ChannelRuntime {
    const fn new() -> Self {
        Self {
            active: false,
            start_us: 0,
            last_level: None,
            transitions: 0,
        }
    }
}

struct GpioEngine {
    channels: [ChannelConfig; GPIO_CHANNEL_COUNT],
    runtime: [ChannelRuntime; GPIO_CHANNEL_COUNT],
}

impl GpioEngine {
    fn new() -> Self {
        Self {
            channels: [ChannelConfig::new(); GPIO_CHANNEL_COUNT],
            runtime: [ChannelRuntime::new(); GPIO_CHANNEL_COUNT],
        }
    }

    fn reset(&mut self) {
        for ch in 0..GPIO_CHANNEL_COUNT {
            self.runtime[ch] = ChannelRuntime::new();
            let _ = self.drive_channel(ch, false);
        }
    }

    fn map_pin(&mut self, ch: usize, pin_index: u32) {
        self.channels[ch].pin_index = Some(pin_index);
        self.runtime[ch].last_level = None;
    }

    fn set_mode(&mut self, ch: usize, mode: ChannelMode) {
        self.channels[ch].mode = mode;
    }

    fn set_timing(&mut self, ch: usize, period_us: u32, high_us: u32) {
        self.channels[ch].period_us = period_us;
        self.channels[ch].high_us = high_us;
        self.channels[ch].repetitions = if period_us == 0 { 0 } else { 1 };
    }

    fn set_pattern(&mut self, ch: usize, bits: u64, bit_len: u8, step_us: u32) {
        self.channels[ch].pattern_bits = Some(bits);
        self.channels[ch].pattern_len = bit_len;
        self.channels[ch].pattern_step_us = step_us;
    }

    fn start_channel(&mut self, ch: usize, now_us: u64) -> bool {
        if self.runtime[ch].active {
            return false;
        }
        self.runtime[ch].active = true;
        self.runtime[ch].start_us = now_us;
        self.runtime[ch].transitions = 0;
        self.runtime[ch].last_level = None;
        let _ = self.update_channel(ch, now_us);
        true
    }

    fn stop_channel(&mut self, ch: usize) -> bool {
        if !self.runtime[ch].active {
            return false;
        }
        self.runtime[ch].active = false;
        self.runtime[ch].transitions = 0;
        self.runtime[ch].start_us = 0;
        self.runtime[ch].last_level = None;
        let _ = self.drive_channel(ch, false);
        true
    }

    fn start_all(&mut self, now_us: u64) -> u32 {
        let mut changed = 0u32;
        for ch in 0..GPIO_CHANNEL_COUNT {
            if self.start_channel(ch, now_us) {
                changed = changed.saturating_add(1);
            }
        }
        changed
    }

    fn stop_all(&mut self) -> u32 {
        let mut changed = 0u32;
        for ch in 0..GPIO_CHANNEL_COUNT {
            if self.stop_channel(ch) {
                changed = changed.saturating_add(1);
            }
        }
        changed
    }

    fn service(&mut self, now_us: u64) {
        for ch in 0..GPIO_CHANNEL_COUNT {
            let _ = self.update_channel(ch, now_us);
        }
    }

    fn channel_level(&mut self, ch: usize, now_us: u64) -> Option<bool> {
        let cfg = self.channels[ch];
        let rt = self.runtime[ch];

        if !rt.active {
            return None;
        }

        let elapsed = now_us.saturating_sub(rt.start_us);
        match cfg.mode {
            ChannelMode::Off => Some(false),
            ChannelMode::High => Some(true),
            ChannelMode::Low => Some(false),
            ChannelMode::Square => {
                if cfg.period_us == 0 {
                    Some(false)
                } else {
                    let period = cfg.period_us as u64;
                    let mut high = cfg.high_us as u64;
                    if high > period {
                        high = period;
                    }
                    Some((elapsed % period) < high)
                }
            }
            ChannelMode::Burst => {
                if cfg.period_us == 0 || cfg.repetitions == 0 {
                    Some(false)
                } else {
                    let total_us = (cfg.period_us as u64).saturating_mul(cfg.repetitions as u64);
                    if elapsed >= total_us {
                        self.runtime[ch].active = false;
                        Some(false)
                    } else {
                        let period = cfg.period_us as u64;
                        let mut high = cfg.high_us as u64;
                        if high > period {
                            high = period;
                        }
                        Some((elapsed % period) < high)
                    }
                }
            }
            ChannelMode::Pattern => {
                let Some(bits) = cfg.pattern_bits else {
                    return Some(false);
                };
                if cfg.pattern_len == 0 || cfg.pattern_step_us == 0 {
                    return Some(false);
                }
                let idx = ((elapsed / cfg.pattern_step_us as u64) % cfg.pattern_len as u64) as u8;
                Some(((bits >> idx) & 0x1) != 0)
            }
        }
    }

    fn update_channel(&mut self, ch: usize, now_us: u64) -> Result<(), &'static str> {
        let Some(level) = self.channel_level(ch, now_us) else {
            return Ok(());
        };

        let rt = &mut self.runtime[ch];
        let changed = rt.last_level != Some(level);
        rt.last_level = Some(level);
        if changed {
            rt.transitions = rt.transitions.saturating_add(1);
            self.drive_channel(ch, level)?;
        }

        Ok(())
    }

    fn drive_channel(&self, ch: usize, high: bool) -> Result<(), &'static str> {
        let Some(pin_index) = self.channels[ch].pin_index else {
            return Ok(());
        };

        let mut pin = Gpio::get_pin(pin_index).map_err(|_| "gpio_get_pin_failed")?;
        let mut out = pin.make_output().map_err(|_| "gpio_output_failed")?;
        if high {
            out.set().map_err(|_| "gpio_set_failed")?;
        } else {
            out.clear().map_err(|_| "gpio_clear_failed")?;
        }
        Ok(())
    }
}

fn print_boot_banner() {
    writeln!(
        Console::writer(),
        "signal-generator-cli v{VERSION} caps=help,caps,reset,start,stop,setfreq,status,gpio radix=dec,0x max_line={MAX_LINE_LEN}"
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

fn parse_hex_u64_ascii(token: &str) -> Result<(u64, u8), &'static str> {
    let raw = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
        .unwrap_or(token);
    if raw.is_empty() {
        return Err("empty_pattern");
    }
    let bits = u64::from_str_radix(raw, 16).map_err(|_| "invalid_hex")?;
    let mut bit_len = (raw.len() * 4) as u8;
    if bit_len == 0 {
        bit_len = 1;
    }
    if bit_len > 64 {
        bit_len = 64;
    }
    Ok((bits, bit_len))
}

fn parse_channel(token: &str) -> Result<usize, &'static str> {
    let index = parse_u32_ascii(token)?;
    if index as usize >= GPIO_CHANNEL_COUNT {
        return Err("channel_out_of_range");
    }
    Ok(index as usize)
}

fn now_us() -> u64 {
    let ticks = Alarm::get_ticks().unwrap_or(0) as u64;
    let freq = Alarm::get_frequency().map(|f| f.0).unwrap_or(1) as u64;
    if freq == 0 {
        return 0;
    }
    ticks.saturating_mul(1_000_000).saturating_div(freq)
}

fn emit_gpio_status(engine: &GpioEngine, ch: usize) {
    let cfg = engine.channels[ch];
    let rt = engine.runtime[ch];
    let pin = cfg.pin_index.map(|v| v as i32).unwrap_or(-1);
    let level = match rt.last_level {
        Some(true) => 1,
        Some(false) => 0,
        None => -1,
    };
    writeln!(
        Console::writer(),
        "OK gpio status ch={} pin={} mode={} active={} period_us={} high_us={} reps={} pattern_bits={} pattern_len={} step_us={} level={} transitions={}",
        ch,
        pin,
        cfg.mode.as_str(),
        if rt.active { 1 } else { 0 },
        cfg.period_us,
        cfg.high_us,
        cfg.repetitions,
        cfg.pattern_bits.unwrap_or(0),
        cfg.pattern_len,
        cfg.pattern_step_us,
        level,
        rt.transitions,
    )
    .unwrap();
}

fn execute_gpio_command<'a, I>(parts: &mut I, state: &mut SignalState)
where
    I: Iterator<Item = &'a str>,
{
    let Some(subcommand) = parts.next() else {
        emit_err("gpio", "ARG", "missing_subcommand");
        return;
    };

    match subcommand {
        "map" => {
            let Some(ch_token) = parts.next() else {
                emit_err("gpio map", "ARG", "missing_channel");
                return;
            };
            let Some(pin_token) = parts.next() else {
                emit_err("gpio map", "ARG", "missing_pin");
                return;
            };
            if parts.next().is_some() {
                emit_err("gpio map", "ARG", "too_many_arguments");
                return;
            }

            let ch = match parse_channel(ch_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("gpio map", "ARG", msg);
                    return;
                }
            };
            let pin = match parse_u32_ascii(pin_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("gpio map", "ARG", msg);
                    return;
                }
            };

            state.gpio.map_pin(ch, pin);
            state.gpio.service(now_us());
            writeln!(Console::writer(), "OK gpio map ch={ch} pin={pin}").unwrap();
        }
        "mode" => {
            let Some(ch_token) = parts.next() else {
                emit_err("gpio mode", "ARG", "missing_channel");
                return;
            };
            let Some(mode_token) = parts.next() else {
                emit_err("gpio mode", "ARG", "missing_mode");
                return;
            };
            if parts.next().is_some() {
                emit_err("gpio mode", "ARG", "too_many_arguments");
                return;
            }

            let ch = match parse_channel(ch_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("gpio mode", "ARG", msg);
                    return;
                }
            };
            let Some(mode) = ChannelMode::parse(mode_token) else {
                emit_err("gpio mode", "ARG", "invalid_mode");
                return;
            };

            state.gpio.set_mode(ch, mode);
            state.gpio.service(now_us());
            writeln!(
                Console::writer(),
                "OK gpio mode ch={ch} mode={}",
                mode.as_str()
            )
            .unwrap();
        }
        "timing" => {
            let Some(ch_token) = parts.next() else {
                emit_err("gpio timing", "ARG", "missing_channel");
                return;
            };
            let Some(period_token) = parts.next() else {
                emit_err("gpio timing", "ARG", "missing_period_us");
                return;
            };

            let high_token = parts.next();
            if parts.next().is_some() {
                emit_err("gpio timing", "ARG", "too_many_arguments");
                return;
            }

            let ch = match parse_channel(ch_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("gpio timing", "ARG", msg);
                    return;
                }
            };
            let period_us = match parse_u32_ascii(period_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("gpio timing", "ARG", msg);
                    return;
                }
            };
            let high_us = match high_token {
                Some(token) => match parse_u32_ascii(token) {
                    Ok(v) => v,
                    Err(msg) => {
                        emit_err("gpio timing", "ARG", msg);
                        return;
                    }
                },
                None => period_us / 2,
            };

            state.gpio.set_timing(ch, period_us, high_us);
            state.gpio.service(now_us());
            writeln!(
                Console::writer(),
                "OK gpio timing ch={ch} period_us={period_us} high_us={high_us}"
            )
            .unwrap();
        }
        "pattern" => {
            let Some(ch_token) = parts.next() else {
                emit_err("gpio pattern", "ARG", "missing_channel");
                return;
            };
            let Some(pattern_token) = parts.next() else {
                emit_err("gpio pattern", "ARG", "missing_hex_bits");
                return;
            };
            let Some(step_token) = parts.next() else {
                emit_err("gpio pattern", "ARG", "missing_step_us");
                return;
            };
            if parts.next().is_some() {
                emit_err("gpio pattern", "ARG", "too_many_arguments");
                return;
            }

            let ch = match parse_channel(ch_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("gpio pattern", "ARG", msg);
                    return;
                }
            };
            let (bits, bit_len) = match parse_hex_u64_ascii(pattern_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("gpio pattern", "ARG", msg);
                    return;
                }
            };
            let step_us = match parse_u32_ascii(step_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("gpio pattern", "ARG", msg);
                    return;
                }
            };

            state.gpio.set_pattern(ch, bits, bit_len, step_us);
            state.gpio.service(now_us());
            writeln!(
                Console::writer(),
                "OK gpio pattern ch={ch} bits=0x{bits:016X} bit_len={bit_len} step_us={step_us}"
            )
            .unwrap();
        }
        "start" => {
            let Some(target) = parts.next() else {
                emit_err("gpio start", "ARG", "missing_channel_or_all");
                return;
            };
            if parts.next().is_some() {
                emit_err("gpio start", "ARG", "too_many_arguments");
                return;
            }

            let now = now_us();
            if target == "all" {
                let changed = state.gpio.start_all(now);
                writeln!(
                    Console::writer(),
                    "OK gpio start target=all changed={changed} active=8"
                )
                .unwrap();
            } else {
                let ch = match parse_channel(target) {
                    Ok(v) => v,
                    Err(msg) => {
                        emit_err("gpio start", "ARG", msg);
                        return;
                    }
                };
                let changed = if state.gpio.start_channel(ch, now) {
                    1
                } else {
                    0
                };
                writeln!(
                    Console::writer(),
                    "OK gpio start target={ch} changed={changed} active=1"
                )
                .unwrap();
            }
        }
        "stop" => {
            let Some(target) = parts.next() else {
                emit_err("gpio stop", "ARG", "missing_channel_or_all");
                return;
            };
            if parts.next().is_some() {
                emit_err("gpio stop", "ARG", "too_many_arguments");
                return;
            }

            if target == "all" {
                let changed = state.gpio.stop_all();
                writeln!(
                    Console::writer(),
                    "OK gpio stop target=all changed={changed} active=0"
                )
                .unwrap();
            } else {
                let ch = match parse_channel(target) {
                    Ok(v) => v,
                    Err(msg) => {
                        emit_err("gpio stop", "ARG", msg);
                        return;
                    }
                };
                let changed = if state.gpio.stop_channel(ch) { 1 } else { 0 };
                writeln!(
                    Console::writer(),
                    "OK gpio stop target={ch} changed={changed} active=0"
                )
                .unwrap();
            }
        }
        "status" => {
            let target = parts.next();
            if parts.next().is_some() {
                emit_err("gpio status", "ARG", "too_many_arguments");
                return;
            }

            state.gpio.service(now_us());
            match target {
                Some(ch_token) => {
                    let ch = match parse_channel(ch_token) {
                        Ok(v) => v,
                        Err(msg) => {
                            emit_err("gpio status", "ARG", msg);
                            return;
                        }
                    };
                    emit_gpio_status(&state.gpio, ch);
                }
                None => {
                    for ch in 0..GPIO_CHANNEL_COUNT {
                        emit_gpio_status(&state.gpio, ch);
                    }
                }
            }
        }
        _ => emit_err("gpio", "UNKNOWN", "unknown_subcommand"),
    }
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
                "commands=help,caps,reset,start,stop,setfreq,status,gpio gpio_subcommands=map,mode,timing,pattern,start,stop,status",
            );
        }
        "caps" => {
            if parts.next().is_some() {
                emit_err("caps", "ARG", "unexpected_arguments");
                return;
            }
            emit_ok(
                "caps",
                "features=help,caps,reset,start,stop,setfreq,status,gpio ascii=1 radix=dec,0x max_line=96 gpio_channels=8 gpio_modes=off,high,low,square,burst,pattern",
            );
        }
        "reset" => {
            if parts.next().is_some() {
                emit_err("reset", "ARG", "unexpected_arguments");
                return;
            }
            state.reset();
            emit_ok("reset", "running=0 frequency_hz=1000 gpio_active=0");
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
            state.gpio.service(now_us());
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
        "gpio" => execute_gpio_command(&mut parts, state),
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
        state.gpio.service(now_us());

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
