use libtock_runtime::TockSyscalls;
use libtock_uart_controller::UartController;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RepeatCount {
    Finite(u32),
    Infinite,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CapabilityResponse {
    Supported,
    Unsupported(&'static str),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Off,
    High,
    Low,
    Square,
    Burst,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Timing {
    pub period_us: u32,
    pub high_us: u32,
    pub repetitions: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ChannelRuntime {
    pub active: bool,
    pub start_us: u64,
    pub last_level: Option<bool>,
    pub transitions: u32,
}

impl ChannelRuntime {
    pub const fn new() -> Self {
        Self {
            active: false,
            start_us: 0,
            last_level: None,
            transitions: 0,
        }
    }
}

pub fn tokenize_command(line: &str) -> core::str::SplitWhitespace<'_> {
    line.split_whitespace()
}

pub fn parse_u32_ascii(token: &str) -> Result<u32, &'static str> {
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

pub fn parse_hex_bytes(token: &str, out: &mut [u8]) -> Result<(usize, bool), &'static str> {
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

pub fn parse_hex_u64_ascii(token: &str) -> Result<(u64, u8), &'static str> {
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

pub fn parse_repeat_count(token: &str) -> Result<RepeatCount, &'static str> {
    if token == "infinite" {
        return Ok(RepeatCount::Infinite);
    }

    match parse_u32_ascii(token)? {
        0 => Err("count_must_be_nonzero"),
        count => Ok(RepeatCount::Finite(count)),
    }
}

pub fn uart_capability_response(secondary_uart_available: bool) -> CapabilityResponse {
    if secondary_uart_available
        && UartController::<TockSyscalls>::is_supported_port(0)
        && UartController::<TockSyscalls>::is_supported_port(1)
    {
        CapabilityResponse::Supported
    } else {
        CapabilityResponse::Unsupported("secondary_uart_not_available")
    }
}

pub fn i2s_capability_response() -> CapabilityResponse {
    CapabilityResponse::Unsupported("i2s_not_available")
}

pub fn gpio_level(
    mode: Mode,
    timing: Timing,
    rt: &mut ChannelRuntime,
    now_us: u64,
) -> Option<bool> {
    if !rt.active {
        return None;
    }

    let elapsed = now_us.saturating_sub(rt.start_us);
    match mode {
        Mode::Off => Some(false),
        Mode::High => Some(true),
        Mode::Low => Some(false),
        Mode::Square => {
            if timing.period_us == 0 {
                Some(false)
            } else {
                let period = timing.period_us as u64;
                let mut high = timing.high_us as u64;
                if high > period {
                    high = period;
                }
                Some((elapsed % period) < high)
            }
        }
        Mode::Burst => {
            if timing.period_us == 0 || timing.repetitions == 0 {
                Some(false)
            } else {
                let total_us = (timing.period_us as u64).saturating_mul(timing.repetitions as u64);
                if elapsed >= total_us {
                    rt.active = false;
                    Some(false)
                } else {
                    let period = timing.period_us as u64;
                    let mut high = timing.high_us as u64;
                    if high > period {
                        high = period;
                    }
                    Some((elapsed % period) < high)
                }
            }
        }
    }
}

pub fn update_gpio_transition(rt: &mut ChannelRuntime, level: Option<bool>) -> bool {
    let Some(level) = level else {
        return false;
    };
    let changed = rt.last_level != Some(level);
    rt.last_level = Some(level);
    if changed {
        rt.transitions = rt.transitions.saturating_add(1);
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenization_collapses_whitespace() {
        let tokens: std::vec::Vec<&str> = tokenize_command("  gpio   mode\t2   square  ").collect();
        assert_eq!(tokens.as_slice(), &["gpio", "mode", "2", "square"]);
    }

    #[test]
    fn numeric_parsing_supports_decimal_and_hex() {
        assert_eq!(parse_u32_ascii("42"), Ok(42));
        assert_eq!(parse_u32_ascii("0x2A"), Ok(42));
        assert_eq!(parse_u32_ascii("0X2a"), Ok(42));
    }

    #[test]
    fn malformed_and_overflow_input_is_rejected() {
        assert_eq!(parse_u32_ascii(""), Err("empty_number"));
        assert_eq!(parse_u32_ascii("hello"), Err("invalid_decimal"));
        assert_eq!(parse_u32_ascii("4294967296"), Err("invalid_decimal"));
        assert_eq!(parse_u32_ascii("0x100000000"), Err("invalid_hex"));

        let mut out = [0u8; 2];
        assert_eq!(parse_hex_bytes("0x0", &mut out), Err("odd_hex_length"));
        assert_eq!(
            parse_hex_bytes("0xZZ", &mut out),
            Err("invalid_hex_payload")
        );
        assert_eq!(parse_hex_u64_ascii("0x"), Err("empty_pattern"));
        assert_eq!(
            parse_hex_u64_ascii("0x1FFFFFFFFFFFFFFFF"),
            Err("invalid_hex")
        );
    }

    #[test]
    fn hex_payload_truncation_is_reported() {
        let mut out = [0u8; 2];
        let parsed = parse_hex_bytes("0xAABBCC", &mut out).unwrap();
        assert_eq!(parsed, (2, true));
        assert_eq!(out, [0xAA, 0xBB]);
    }

    #[test]
    fn gpio_mode_transitions_follow_timing() {
        let mut rt = ChannelRuntime::new();
        rt.active = true;
        rt.start_us = 1_000;

        let timing = Timing {
            period_us: 10,
            high_us: 4,
            repetitions: 0,
        };

        let level = gpio_level(Mode::Square, timing, &mut rt, 1_000);
        assert!(update_gpio_transition(&mut rt, level));
        assert_eq!(rt.last_level, Some(true));
        assert_eq!(rt.transitions, 1);

        let level = gpio_level(Mode::Square, timing, &mut rt, 1_005);
        assert!(update_gpio_transition(&mut rt, level));
        assert_eq!(rt.last_level, Some(false));
        assert_eq!(rt.transitions, 2);
    }

    #[test]
    fn burst_mode_auto_deactivates_after_repetitions() {
        let mut rt = ChannelRuntime::new();
        rt.active = true;
        let timing = Timing {
            period_us: 10,
            high_us: 5,
            repetitions: 2,
        };

        assert_eq!(gpio_level(Mode::Burst, timing, &mut rt, 19), Some(false));
        assert!(rt.active);

        assert_eq!(gpio_level(Mode::Burst, timing, &mut rt, 20), Some(false));
        assert!(!rt.active);
    }

    #[test]
    fn repeat_and_stop_values_parse_as_expected() {
        assert_eq!(parse_repeat_count("infinite"), Ok(RepeatCount::Infinite));
        assert_eq!(parse_repeat_count("3"), Ok(RepeatCount::Finite(3)));
        assert_eq!(parse_repeat_count("0"), Err("count_must_be_nonzero"));
    }

    #[test]
    fn capability_gates_uart_and_i2s() {
        assert_eq!(
            uart_capability_response(true),
            CapabilityResponse::Supported
        );
        assert_eq!(
            uart_capability_response(false),
            CapabilityResponse::Unsupported("secondary_uart_not_available")
        );
        assert_eq!(
            i2s_capability_response(),
            CapabilityResponse::Unsupported("i2s_not_available")
        );
    }
}
