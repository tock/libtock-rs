//! Tempoary formatting functions until format! is fixed

use syscalls;

pub fn output_number(value: u32) {
    let mut out: [u8; 11] = [32; 11];
    write_u32_into_array(&mut out, value as u32, 0x10_00_00_00, 0x10);

    unsafe {
        let handle = syscalls::allow(1, 1, &mut out);
        syscalls::command(1, 1, 10, 0);
        handle.unwrap();
    }
}
pub fn write_u32_into_array(result: &mut [u8; 11], value: u32, start: u32, base: u32) {
    let mut scanning = start;
    let mut remainder = value;
    let mut counter = 0;
    result[0] = '0' as u8;
    result[1] = 'x' as u8;
    result[10] = '\n' as u8;

    while scanning > 0 {
        let digit = remainder / scanning;
        result[counter + 2] = render_digit(digit as u8) as u8;

        remainder = remainder % scanning;
        scanning = scanning / base;
        counter += 1;
    }
}

fn render_digit(digit: u8) -> char {
    if digit < 10 {
        ('0' as u8 + digit) as char
    } else {
        ('a' as u8 + digit - 10) as char
    }
}
