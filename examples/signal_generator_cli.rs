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
use libtock::spi_controller::SpiController;

set_main! {main}
stack_size! {0x600}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_LINE_LEN: usize = 96;
const GPIO_CHANNEL_COUNT: usize = 8;
const SPI_BUF_MAX: usize = 64;
const UART_BUF_MAX: usize = 64;
const UART_SECONDARY_SUPPORTED: bool = cfg!(feature = "secondary_uart");

struct SignalState {
    running: bool,
    frequency_hz: u32,
    gpio: GpioEngine,
    spi: SpiState,
    uart: UartState,
}

impl SignalState {
    fn new() -> Self {
        Self {
            running: false,
            frequency_hz: 1_000,
            gpio: GpioEngine::new(),
            spi: SpiState::new(),
            uart: UartState::new(),
        }
    }

    fn reset(&mut self) {
        self.running = false;
        self.frequency_hz = 1_000;
        self.gpio.reset();
        self.spi = SpiState::new();
        self.uart = UartState::new();
    }
}

#[derive(Copy, Clone)]
enum UartParity {
    None,
    Even,
    Odd,
}

impl UartParity {
    fn as_str(self) -> &'static str {
        match self {
            UartParity::None => "none",
            UartParity::Even => "even",
            UartParity::Odd => "odd",
        }
    }

    fn parse(token: &str) -> Option<Self> {
        match token {
            "none" => Some(UartParity::None),
            "even" => Some(UartParity::Even),
            "odd" => Some(UartParity::Odd),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
enum UartFormat {
    Hex,
    Ascii,
}

impl UartFormat {
    fn as_str(self) -> &'static str {
        match self {
            UartFormat::Hex => "hex",
            UartFormat::Ascii => "ascii",
        }
    }
}

#[derive(Copy, Clone)]
struct UartState {
    port: u32,
    baud: u32,
    data_bits: u8,
    parity: UartParity,
    stop_bits: u8,
    repeat_active: bool,
    repeat_interval_us: u32,
    repeat_count: u32,
    repeat_infinite: bool,
    last_payload: [u8; UART_BUF_MAX],
    last_payload_len: usize,
    last_payload_format: UartFormat,
}

impl UartState {
    const fn new() -> Self {
        Self {
            port: 1,
            baud: 115_200,
            data_bits: 8,
            parity: UartParity::None,
            stop_bits: 1,
            repeat_active: false,
            repeat_interval_us: 0,
            repeat_count: 0,
            repeat_infinite: false,
            last_payload: [0; UART_BUF_MAX],
            last_payload_len: 0,
            last_payload_format: UartFormat::Hex,
        }
    }
}

#[derive(Copy, Clone)]
enum SpiMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

impl SpiMode {
    fn as_str(&self) -> &'static str {
        match self {
            SpiMode::Mode0 => "mode0",
            SpiMode::Mode1 => "mode1",
            SpiMode::Mode2 => "mode2",
            SpiMode::Mode3 => "mode3",
        }
    }

    fn parse(token: &str) -> Option<Self> {
        match token {
            "mode0" => Some(SpiMode::Mode0),
            "mode1" => Some(SpiMode::Mode1),
            "mode2" => Some(SpiMode::Mode2),
            "mode3" => Some(SpiMode::Mode3),
            _ => None,
        }
    }

    fn phase(self) -> bool {
        matches!(self, SpiMode::Mode1 | SpiMode::Mode3)
    }

    fn polarity(self) -> bool {
        matches!(self, SpiMode::Mode2 | SpiMode::Mode3)
    }
}

#[derive(Copy, Clone)]
struct SpiState {
    baud_hz: u32,
    mode: SpiMode,
    cs: u32,
    repeat_active: bool,
    last_tx: [u8; SPI_BUF_MAX],
    last_tx_len: usize,
    last_rx: [u8; SPI_BUF_MAX],
    last_rx_len: usize,
}

impl SpiState {
    const fn new() -> Self {
        Self {
            baud_hz: 1_000_000,
            mode: SpiMode::Mode0,
            cs: 0,
            repeat_active: false,
            last_tx: [0; SPI_BUF_MAX],
            last_tx_len: 0,
            last_rx: [0; SPI_BUF_MAX],
            last_rx_len: 0,
        }
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
        "signal-generator-cli v{VERSION} caps=help,caps,reset,start,stop,setfreq,status,gpio,spi,uart radix=dec,0x max_line={MAX_LINE_LEN}"
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

fn secondary_uart_available() -> bool {
    UART_SECONDARY_SUPPORTED
}

fn emit_uart_unsupported() {
    emit_err("uart", "UNSUPPORTED", "secondary_uart_not_available");
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

fn parse_hex_bytes(token: &str, out: &mut [u8]) -> Result<(usize, bool), &'static str> {
    let raw = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
        .unwrap_or(token);
    if raw.is_empty() {
        return Err("empty_hex_payload");
    }
    if !raw.len().is_multiple_of(2) {
        return Err("odd_hex_length");
    }

    let mut len = 0usize;
    let mut truncated = false;
    let bytes = raw.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        if len >= out.len() {
            truncated = true;
            break;
        }
        let chunk = &raw[idx..idx + 2];
        out[len] = u8::from_str_radix(chunk, 16).map_err(|_| "invalid_hex_payload")?;
        len += 1;
        idx += 2;
    }
    Ok((len, truncated))
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

fn sleep_us(interval_us: u32) {
    if interval_us == 0 {
        return;
    }
    let Ok(freq) = Alarm::get_frequency() else {
        return;
    };
    let ticks = ((interval_us as u64)
        .saturating_mul(freq.0 as u64)
        .saturating_add(999_999))
    .saturating_div(1_000_000);
    if ticks == 0 {
        return;
    }
    let ticks = ticks.min(u32::MAX as u64) as u32;
    let _ = Alarm::sleep_for(libtock::alarm::Ticks(ticks));
}

fn emit_hex_bytes(bytes: &[u8]) {
    write!(Console::writer(), "0x").unwrap();
    for byte in bytes {
        write!(Console::writer(), "{byte:02x}").unwrap();
    }
}

fn parse_uart_port(token: &str) -> Result<u32, &'static str> {
    let port = parse_u32_ascii(token)?;
    if port == 0 {
        return Err("port_must_be_nonzero");
    }
    Ok(port)
}

fn parse_uart_payload_ascii<'a, I>(parts: I, out: &mut [u8]) -> (usize, bool)
where
    I: Iterator<Item = &'a str>,
{
    let mut written = 0usize;
    let mut truncated = false;
    let mut first = true;
    for token in parts {
        if !first {
            if written < out.len() {
                out[written] = b' ';
                written += 1;
            } else {
                truncated = true;
            }
        }
        first = false;
        for byte in token.bytes() {
            if written < out.len() {
                out[written] = byte;
                written += 1;
            } else {
                truncated = true;
            }
        }
    }
    (written, truncated)
}

fn parse_uart_payload_hex<'a, I>(parts: I, out: &mut [u8]) -> Result<(usize, bool), &'static str>
where
    I: Iterator<Item = &'a str>,
{
    let mut written = 0usize;
    let mut truncated = false;
    let mut saw_token = false;
    for token in parts {
        saw_token = true;
        let (len, token_truncated) = parse_hex_bytes(token, &mut out[written..])?;
        written += len;
        if token_truncated || written == out.len() {
            truncated = token_truncated;
        }
    }
    if !saw_token {
        return Err("missing_payload");
    }
    Ok((written, truncated))
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

fn execute_spi_command<'a, I>(parts: &mut I, state: &mut SignalState)
where
    I: Iterator<Item = &'a str>,
{
    let Some(subcommand) = parts.next() else {
        emit_err("spi", "ARG", "missing_subcommand");
        return;
    };

    match subcommand {
        "cfg" => {
            let Some(hz_token) = parts.next() else {
                emit_err("spi cfg", "ARG", "missing_hz");
                return;
            };
            let Some(mode_token) = parts.next() else {
                emit_err("spi cfg", "ARG", "missing_mode");
                return;
            };
            let Some(cs_token) = parts.next() else {
                emit_err("spi cfg", "ARG", "missing_cs");
                return;
            };
            if parts.next().is_some() {
                emit_err("spi cfg", "ARG", "too_many_arguments");
                return;
            }

            let hz = match parse_u32_ascii(hz_token) {
                Ok(v) if v > 0 => v,
                Ok(_) => {
                    emit_err("spi cfg", "RANGE", "hz_must_be_nonzero");
                    return;
                }
                Err(msg) => {
                    emit_err("spi cfg", "ARG", msg);
                    return;
                }
            };
            let Some(mode) = SpiMode::parse(mode_token) else {
                emit_err("spi cfg", "ARG", "invalid_mode");
                return;
            };
            let cs = match parse_u32_ascii(cs_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("spi cfg", "ARG", msg);
                    return;
                }
            };

            if let Err(why) = SpiController::set_baud_rate(hz) {
                writeln!(Console::writer(), "ERR spi cfg IO set_baud_failed_{why:?}").unwrap();
                return;
            }
            if let Err(why) = SpiController::set_polarity(mode.polarity()) {
                writeln!(
                    Console::writer(),
                    "ERR spi cfg IO set_polarity_failed_{why:?}"
                )
                .unwrap();
                return;
            }
            if let Err(why) = SpiController::set_phase(mode.phase()) {
                writeln!(Console::writer(), "ERR spi cfg IO set_phase_failed_{why:?}").unwrap();
                return;
            }

            state.spi.baud_hz = hz;
            state.spi.mode = mode;
            state.spi.cs = cs;
            writeln!(
                Console::writer(),
                "OK spi cfg hz={} mode={} cs={}",
                state.spi.baud_hz,
                state.spi.mode.as_str(),
                state.spi.cs
            )
            .unwrap();
        }
        "tx" => {
            let Some(payload_token) = parts.next() else {
                emit_err("spi tx", "ARG", "missing_hex_bytes");
                return;
            };
            if parts.next().is_some() {
                emit_err("spi tx", "ARG", "too_many_arguments");
                return;
            }

            let (tx_len, truncated) = match parse_hex_bytes(payload_token, &mut state.spi.last_tx) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("spi tx", "ARG", msg);
                    return;
                }
            };
            if tx_len == 0 {
                emit_err("spi tx", "RANGE", "empty_tx_payload");
                return;
            }

            if let Err(why) = SpiController::spi_controller_write_sync_with_chip_select(
                &state.spi.last_tx[..tx_len],
                tx_len as u32,
                state.spi.cs,
            ) {
                writeln!(Console::writer(), "ERR spi tx IO write_failed_{why:?}").unwrap();
                return;
            }

            state.spi.last_tx_len = tx_len;
            state.spi.last_rx_len = 0;
            write!(
                Console::writer(),
                "OK spi tx tx_len={} truncated={} tx=",
                tx_len,
                if truncated { 1 } else { 0 }
            )
            .unwrap();
            emit_hex_bytes(&state.spi.last_tx[..tx_len]);
            writeln!(Console::writer()).unwrap();
        }
        "txrx" => {
            let Some(payload_token) = parts.next() else {
                emit_err("spi txrx", "ARG", "missing_hex_bytes");
                return;
            };
            let Some(rx_len_token) = parts.next() else {
                emit_err("spi txrx", "ARG", "missing_rx_len");
                return;
            };
            if parts.next().is_some() {
                emit_err("spi txrx", "ARG", "too_many_arguments");
                return;
            }

            let (tx_len, truncated) = match parse_hex_bytes(payload_token, &mut state.spi.last_tx) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("spi txrx", "ARG", msg);
                    return;
                }
            };
            let rx_requested = match parse_u32_ascii(rx_len_token) {
                Ok(v) => v as usize,
                Err(msg) => {
                    emit_err("spi txrx", "ARG", msg);
                    return;
                }
            };
            let rx_len = rx_requested.min(SPI_BUF_MAX);

            if tx_len == 0 {
                emit_err("spi txrx", "RANGE", "empty_tx_payload");
                return;
            }

            if let Err(why) = SpiController::spi_controller_write_sync_with_chip_select(
                &state.spi.last_tx[..tx_len],
                tx_len as u32,
                state.spi.cs,
            ) {
                writeln!(Console::writer(), "ERR spi txrx IO write_failed_{why:?}").unwrap();
                return;
            }
            if let Err(why) = SpiController::spi_controller_read_sync_with_chip_select(
                &mut state.spi.last_rx[..rx_len],
                rx_len as u32,
                state.spi.cs,
            ) {
                writeln!(Console::writer(), "ERR spi txrx IO read_failed_{why:?}").unwrap();
                return;
            }

            state.spi.last_tx_len = tx_len;
            state.spi.last_rx_len = rx_len;
            write!(
                Console::writer(),
                "OK spi txrx tx_len={} rx_len={} rx_requested={} truncated={} tx=",
                tx_len,
                rx_len,
                rx_requested,
                if truncated || rx_requested > SPI_BUF_MAX {
                    1
                } else {
                    0
                }
            )
            .unwrap();
            emit_hex_bytes(&state.spi.last_tx[..tx_len]);
            write!(Console::writer(), " rx=").unwrap();
            emit_hex_bytes(&state.spi.last_rx[..rx_len]);
            writeln!(Console::writer()).unwrap();
        }
        "repeat" => {
            let Some(payload_token) = parts.next() else {
                emit_err("spi repeat", "ARG", "missing_hex_bytes");
                return;
            };
            let Some(count_token) = parts.next() else {
                emit_err("spi repeat", "ARG", "missing_count");
                return;
            };
            let Some(interval_token) = parts.next() else {
                emit_err("spi repeat", "ARG", "missing_interval_us");
                return;
            };
            if parts.next().is_some() {
                emit_err("spi repeat", "ARG", "too_many_arguments");
                return;
            }

            let (tx_len, truncated) = match parse_hex_bytes(payload_token, &mut state.spi.last_tx) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("spi repeat", "ARG", msg);
                    return;
                }
            };
            let count = match parse_u32_ascii(count_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("spi repeat", "ARG", msg);
                    return;
                }
            };
            let interval_us = match parse_u32_ascii(interval_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("spi repeat", "ARG", msg);
                    return;
                }
            };

            if tx_len == 0 {
                emit_err("spi repeat", "RANGE", "empty_tx_payload");
                return;
            }
            if count == 0 {
                emit_err("spi repeat", "RANGE", "count_must_be_nonzero");
                return;
            }

            let mut sent = 0u32;
            state.spi.repeat_active = true;
            while sent < count && state.spi.repeat_active {
                if let Err(why) = SpiController::spi_controller_write_sync_with_chip_select(
                    &state.spi.last_tx[..tx_len],
                    tx_len as u32,
                    state.spi.cs,
                ) {
                    state.spi.repeat_active = false;
                    writeln!(Console::writer(), "ERR spi repeat IO write_failed_{why:?}").unwrap();
                    return;
                }
                sent = sent.saturating_add(1);
                if sent < count {
                    sleep_us(interval_us);
                }
            }
            state.spi.repeat_active = false;
            state.spi.last_tx_len = tx_len;
            state.spi.last_rx_len = 0;
            write!(
                Console::writer(),
                "OK spi repeat count={} sent={} interval_us={} truncated={} tx=",
                count,
                sent,
                interval_us,
                if truncated { 1 } else { 0 }
            )
            .unwrap();
            emit_hex_bytes(&state.spi.last_tx[..tx_len]);
            writeln!(Console::writer()).unwrap();
        }
        "stop" => {
            if parts.next().is_some() {
                emit_err("spi stop", "ARG", "unexpected_arguments");
                return;
            }
            state.spi.repeat_active = false;
            emit_ok("spi stop", "active=0");
        }
        "status" => {
            if parts.next().is_some() {
                emit_err("spi status", "ARG", "unexpected_arguments");
                return;
            }
            let baud = SpiController::get_baud_rate().unwrap_or(state.spi.baud_hz);
            let mode = match (
                SpiController::get_polarity().unwrap_or(state.spi.mode.polarity()),
                SpiController::get_phase().unwrap_or(state.spi.mode.phase()),
            ) {
                (false, false) => SpiMode::Mode0,
                (false, true) => SpiMode::Mode1,
                (true, false) => SpiMode::Mode2,
                (true, true) => SpiMode::Mode3,
            };
            state.spi.baud_hz = baud;
            state.spi.mode = mode;
            write!(
                Console::writer(),
                "OK spi status active={} hz={} mode={} cs={} tx_len={} rx_len={} tx=",
                if state.spi.repeat_active { 1 } else { 0 },
                state.spi.baud_hz,
                state.spi.mode.as_str(),
                state.spi.cs,
                state.spi.last_tx_len,
                state.spi.last_rx_len
            )
            .unwrap();
            emit_hex_bytes(&state.spi.last_tx[..state.spi.last_tx_len]);
            write!(Console::writer(), " rx=").unwrap();
            emit_hex_bytes(&state.spi.last_rx[..state.spi.last_rx_len]);
            writeln!(Console::writer()).unwrap();
        }
        _ => emit_err("spi", "UNKNOWN", "unknown_subcommand"),
    }
}

fn execute_uart_command<'a, I>(parts: &mut I, state: &mut SignalState)
where
    I: Iterator<Item = &'a str>,
{
    if !secondary_uart_available() {
        emit_uart_unsupported();
        return;
    }

    let Some(subcommand) = parts.next() else {
        emit_err("uart", "ARG", "missing_subcommand");
        return;
    };

    match subcommand {
        "cfg" => {
            let Some(port_token) = parts.next() else {
                emit_err("uart cfg", "ARG", "missing_port");
                return;
            };
            let Some(baud_token) = parts.next() else {
                emit_err("uart cfg", "ARG", "missing_baud");
                return;
            };
            let Some(data_bits_token) = parts.next() else {
                emit_err("uart cfg", "ARG", "missing_data_bits");
                return;
            };
            let Some(parity_token) = parts.next() else {
                emit_err("uart cfg", "ARG", "missing_parity");
                return;
            };
            let Some(stop_bits_token) = parts.next() else {
                emit_err("uart cfg", "ARG", "missing_stop_bits");
                return;
            };
            if parts.next().is_some() {
                emit_err("uart cfg", "ARG", "too_many_arguments");
                return;
            }

            let port = match parse_uart_port(port_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("uart cfg", "ARG", msg);
                    return;
                }
            };
            let baud = match parse_u32_ascii(baud_token) {
                Ok(v) if v > 0 => v,
                Ok(_) => {
                    emit_err("uart cfg", "RANGE", "baud_must_be_nonzero");
                    return;
                }
                Err(msg) => {
                    emit_err("uart cfg", "ARG", msg);
                    return;
                }
            };
            let data_bits = match parse_u32_ascii(data_bits_token) {
                Ok(v @ 5..=8) => v as u8,
                Ok(_) => {
                    emit_err("uart cfg", "RANGE", "data_bits_must_be_5_to_8");
                    return;
                }
                Err(msg) => {
                    emit_err("uart cfg", "ARG", msg);
                    return;
                }
            };
            let Some(parity) = UartParity::parse(parity_token) else {
                emit_err("uart cfg", "ARG", "invalid_parity");
                return;
            };
            let stop_bits = match parse_u32_ascii(stop_bits_token) {
                Ok(v @ 1..=2) => v as u8,
                Ok(_) => {
                    emit_err("uart cfg", "RANGE", "stop_bits_must_be_1_or_2");
                    return;
                }
                Err(msg) => {
                    emit_err("uart cfg", "ARG", msg);
                    return;
                }
            };

            state.uart.port = port;
            state.uart.baud = baud;
            state.uart.data_bits = data_bits;
            state.uart.parity = parity;
            state.uart.stop_bits = stop_bits;
            writeln!(
                Console::writer(),
                "OK uart cfg port={} baud={} data_bits={} parity={} stop_bits={}",
                state.uart.port,
                state.uart.baud,
                state.uart.data_bits,
                state.uart.parity.as_str(),
                state.uart.stop_bits
            )
            .unwrap();
        }
        "tx" => {
            let Some(port_token) = parts.next() else {
                emit_err("uart tx", "ARG", "missing_port");
                return;
            };
            let Some(format_token) = parts.next() else {
                emit_err("uart tx", "ARG", "missing_payload_format");
                return;
            };
            let port = match parse_uart_port(port_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("uart tx", "ARG", msg);
                    return;
                }
            };
            let (format, len, truncated) = match format_token {
                "hex" => match parse_uart_payload_hex(parts.by_ref(), &mut state.uart.last_payload)
                {
                    Ok((len, truncated)) => (UartFormat::Hex, len, truncated),
                    Err(msg) => {
                        emit_err("uart tx", "ARG", msg);
                        return;
                    }
                },
                "ascii" => {
                    let (len, truncated) =
                        parse_uart_payload_ascii(parts.by_ref(), &mut state.uart.last_payload);
                    (UartFormat::Ascii, len, truncated)
                }
                _ => {
                    emit_err("uart tx", "ARG", "invalid_payload_format");
                    return;
                }
            };

            if len == 0 {
                emit_err("uart tx", "RANGE", "empty_payload");
                return;
            }

            state.uart.port = port;
            state.uart.last_payload_len = len;
            state.uart.last_payload_format = format;
            write!(
                Console::writer(),
                "OK uart tx port={} len={} format={} truncated={} payload=",
                state.uart.port,
                state.uart.last_payload_len,
                state.uart.last_payload_format.as_str(),
                if truncated { 1 } else { 0 }
            )
            .unwrap();
            emit_hex_bytes(&state.uart.last_payload[..state.uart.last_payload_len]);
            writeln!(Console::writer()).unwrap();
        }
        "repeat" => {
            let Some(port_token) = parts.next() else {
                emit_err("uart repeat", "ARG", "missing_port");
                return;
            };
            let Some(payload_token) = parts.next() else {
                emit_err("uart repeat", "ARG", "missing_payload");
                return;
            };
            let Some(count_token) = parts.next() else {
                emit_err("uart repeat", "ARG", "missing_count_or_infinite");
                return;
            };
            let Some(interval_token) = parts.next() else {
                emit_err("uart repeat", "ARG", "missing_interval_us");
                return;
            };
            if parts.next().is_some() {
                emit_err("uart repeat", "ARG", "too_many_arguments");
                return;
            }

            let port = match parse_uart_port(port_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("uart repeat", "ARG", msg);
                    return;
                }
            };
            let (len, truncated) =
                if payload_token.starts_with("0x") || payload_token.starts_with("0X") {
                    match parse_hex_bytes(payload_token, &mut state.uart.last_payload) {
                        Ok(v) => v,
                        Err(msg) => {
                            emit_err("uart repeat", "ARG", msg);
                            return;
                        }
                    }
                } else {
                    let (len, truncated) = parse_uart_payload_ascii(
                        core::iter::once(payload_token),
                        &mut state.uart.last_payload,
                    );
                    (len, truncated)
                };
            if len == 0 {
                emit_err("uart repeat", "RANGE", "empty_payload");
                return;
            }
            let (repeat_infinite, repeat_count) = if count_token == "infinite" {
                (true, 0)
            } else {
                match parse_u32_ascii(count_token) {
                    Ok(v) if v > 0 => (false, v),
                    Ok(_) => {
                        emit_err("uart repeat", "RANGE", "count_must_be_nonzero");
                        return;
                    }
                    Err(msg) => {
                        emit_err("uart repeat", "ARG", msg);
                        return;
                    }
                }
            };
            let interval_us = match parse_u32_ascii(interval_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("uart repeat", "ARG", msg);
                    return;
                }
            };

            state.uart.port = port;
            state.uart.repeat_active = true;
            state.uart.repeat_infinite = repeat_infinite;
            state.uart.repeat_count = repeat_count;
            state.uart.repeat_interval_us = interval_us;
            state.uart.last_payload_len = len;
            state.uart.last_payload_format =
                if payload_token.starts_with("0x") || payload_token.starts_with("0X") {
                    UartFormat::Hex
                } else {
                    UartFormat::Ascii
                };

            write!(
                Console::writer(),
                "OK uart repeat port={} active=1 count={} infinite={} interval_us={} truncated={} payload=",
                state.uart.port,
                state.uart.repeat_count,
                if state.uart.repeat_infinite { 1 } else { 0 },
                state.uart.repeat_interval_us,
                if truncated { 1 } else { 0 }
            )
            .unwrap();
            emit_hex_bytes(&state.uart.last_payload[..state.uart.last_payload_len]);
            writeln!(Console::writer()).unwrap();
        }
        "stop" => {
            let Some(port_token) = parts.next() else {
                emit_err("uart stop", "ARG", "missing_port");
                return;
            };
            if parts.next().is_some() {
                emit_err("uart stop", "ARG", "too_many_arguments");
                return;
            }
            let port = match parse_uart_port(port_token) {
                Ok(v) => v,
                Err(msg) => {
                    emit_err("uart stop", "ARG", msg);
                    return;
                }
            };
            state.uart.port = port;
            state.uart.repeat_active = false;
            state.uart.repeat_count = 0;
            state.uart.repeat_infinite = false;
            emit_ok("uart stop", "active=0");
        }
        "status" => {
            let port = match parts.next() {
                Some(token) => match parse_uart_port(token) {
                    Ok(v) => v,
                    Err(msg) => {
                        emit_err("uart status", "ARG", msg);
                        return;
                    }
                },
                None => state.uart.port,
            };
            if parts.next().is_some() {
                emit_err("uart status", "ARG", "too_many_arguments");
                return;
            }
            state.uart.port = port;
            write!(
                Console::writer(),
                "OK uart status port={} active={} baud={} data_bits={} parity={} stop_bits={} repeat_count={} repeat_infinite={} interval_us={} payload_len={} payload_format={} payload=",
                state.uart.port,
                if state.uart.repeat_active { 1 } else { 0 },
                state.uart.baud,
                state.uart.data_bits,
                state.uart.parity.as_str(),
                state.uart.stop_bits,
                state.uart.repeat_count,
                if state.uart.repeat_infinite { 1 } else { 0 },
                state.uart.repeat_interval_us,
                state.uart.last_payload_len,
                state.uart.last_payload_format.as_str(),
            )
            .unwrap();
            emit_hex_bytes(&state.uart.last_payload[..state.uart.last_payload_len]);
            writeln!(Console::writer()).unwrap();
        }
        _ => emit_err("uart", "UNKNOWN", "unknown_subcommand"),
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
                "commands=help,caps,reset,start,stop,setfreq,status,gpio,spi,uart gpio_subcommands=map,mode,timing,pattern,start,stop,status spi_subcommands=cfg,tx,txrx,repeat,stop,status uart_subcommands=cfg,tx,repeat,stop,status",
            );
        }
        "caps" => {
            if parts.next().is_some() {
                emit_err("caps", "ARG", "unexpected_arguments");
                return;
            }
            writeln!(
                Console::writer(),
                "OK caps features=help,caps,reset,start,stop,setfreq,status,gpio,spi,uart ascii=1 radix=dec,0x max_line=96 gpio_channels=8 gpio_modes=off,high,low,square,burst,pattern spi_buf_max=64 spi_modes=mode0,mode1,mode2,mode3 uart_secondary={} uart_buf_max=64",
                if secondary_uart_available() { "true" } else { "false" }
            )
            .unwrap();
        }
        "reset" => {
            if parts.next().is_some() {
                emit_err("reset", "ARG", "unexpected_arguments");
                return;
            }
            state.reset();
            emit_ok(
                "reset",
                "running=0 frequency_hz=1000 gpio_active=0 uart_active=0",
            );
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
        "spi" => execute_spi_command(&mut parts, state),
        "uart" => execute_uart_command(&mut parts, state),
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
