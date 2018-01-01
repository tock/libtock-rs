//! Tempoary formatting functions until format! is fixed

use alloc::string::String;

pub fn u32_as_decimal(value: u32) -> String {
    let mut result = String::new();
    write_u32_as_any_base(&mut result, value, 1_000_000_000, 10);
    result
}

fn write_u32_as_any_base(result: &mut String, value: u32, start: u32, base: u32) {
    let mut scanning = start;
    let mut remaining = value;
    while scanning > 0 {
        let digit = (remaining / scanning) as u8;
        remaining = remaining % scanning;
        scanning = scanning / base;
        result.push(('0' as u8 + digit) as char);
    }
}
