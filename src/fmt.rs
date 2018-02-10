//! Tempoary formatting functions until format! is fixed

use alloc::string::String;

pub fn u32_as_decimal(value: u32) -> String {
    let mut result = String::new();
    write_u32_as_any_base(&mut result, value, 1_000_000_000, 10);
    result
}

pub fn i32_as_decimal(value: i32) -> String {
    let mut result = String::new();
    let mut value = value;
    if value < 0 {
        result.push_str("-");
        value = -value;
    }
    write_u32_as_any_base(&mut result, value as u32, 1_000_000_000, 10);
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

pub fn convert_le(number: u16) -> [u8; 2] {
    let mut array: [u8; 2] = [0; 2];
    array[0] = number as u8;
    array[1] = (number >> 8) as u8;
    array
}

#[cfg(test)]
mod test {
    use fmt::*;

    #[test]
    pub fn digits_are_correctly_rendered_in_decimal() {
        assert_eq!(u32_as_decimal(123), String::from("0000000123"));
        assert_eq!(u32_as_decimal(2000000123), String::from("2000000123"));
    }

    #[test]
    pub fn negative_numbers_are_rendered() {
        assert_eq!(i32_as_decimal(-123), String::from("-0000000123"));
    }

    #[test]
    pub fn digits_are_correctly_rendered_in_hex() {
        assert_eq!(u32_as_hex(0x1000_0000), String::from("0x10000000"));
        assert_eq!(u32_as_hex(0x1000_3000), String::from("0x10003000"));
        assert_eq!(u32_as_hex(0x0000_0000), String::from("0x00000000"));
    }

    #[test]
    pub fn number_are_rendered_correctly() {
        assert_eq!(convert_le(0x00FF), [255, 0]);
        assert_eq!(convert_le(0x0100), [0, 1]);
        assert_eq!(convert_le(0x01FF), [255, 1]);
    }
}
