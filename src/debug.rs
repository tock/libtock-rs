//! Heapless debugging functions for Tock troubleshooting

use crate::console::Console;

pub fn print_as_hex(value: usize) {
    let mut buffer = [b'\n'; 11];
    write_as_hex(&mut buffer, value);
    Console::new().write_bytes(&buffer);
}

pub fn print_stack_pointer() {
    let stack_pointer;
    unsafe { asm!("mov $0, sp" : "=r"(stack_pointer) : : : "volatile") };

    let mut buffer = [b'\n'; 15];
    buffer[0..4].clone_from_slice(b"SP: ");
    write_as_hex(&mut buffer[4..15], stack_pointer);
    Console::new().write_bytes(&buffer);
}

#[inline(always)] // Initial stack size is too small (64 bytes currently)
pub fn dump_address(address: *const usize) {
    let mut buffer = [b'\n'; 23];
    write_as_hex(&mut buffer[0..10], address as usize);
    buffer[10..12].clone_from_slice(b": ");
    write_as_hex(&mut buffer[12..22], unsafe { *address });
    Console::new().write_bytes(&buffer);
}

fn write_as_hex(buffer: &mut [u8], value: usize) {
    write_formatted(buffer, value, 0x10_00_00_00, 0x10);
}

fn write_formatted(buffer: &mut [u8], value: usize, start: usize, base: usize) {
    let mut scanning = start;
    let mut remainder = value;
    let mut position = 2;
    buffer[0..2].clone_from_slice(b"0x");

    while scanning > 0 {
        let digit = remainder / scanning;
        buffer[position] = render_digit(digit as u8) as u8;

        remainder %= scanning;
        scanning /= base;
        position += 1;
    }
}

fn render_digit(digit: u8) -> char {
    if digit < 10 {
        (b'0' + digit) as char
    } else {
        (b'a' + digit - 10) as char
    }
}
