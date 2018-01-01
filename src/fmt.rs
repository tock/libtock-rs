//! Tempoary formatting functions until format! is fixed

use alloc::string::String;

pub fn u32_as_decimal(value: u32) -> String {
    let mut result = String::new();
    write_u32_as_any_base(&mut result, value, 1_000_000_000, 10);
    result
}

pub fn u32_as_hex(value: u32) -> String {
    let mut result = String::new();
    result.push_str("0x");
    write_u32_as_any_base(&mut result, value, 0x1000_0000, 0x10);
    result
}

fn write_u32_as_any_base(result: &mut String, value: u32, start: u32, base: u32) {
    let mut scanning = start;
    let mut remaining = value;
    while scanning > 0 {
        let digit = remaining / scanning;
        result.push(render_digit(digit as u8));

        remaining = remaining % scanning;
        scanning = scanning / base;
    }
}

fn render_digit(digit: u8) -> char {
    if digit < 10 {
        ('0' as u8 + digit) as char
    } else {
        ('a' as u8 + digit - 10) as char
    }
}
