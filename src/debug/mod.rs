//! Heapless debugging functions for Tock troubleshooting

mod low_level_debug;
pub use low_level_debug::*;

use crate::console::Console;

pub fn println() {
    let buffer = [b'\n'];
    Console::new().write(&buffer);
}

pub fn print_as_hex(value: usize) {
    let mut buffer = [b'\n'; 11];
    write_as_hex(&mut buffer, value);
    Console::new().write(buffer);
}

#[cfg(target_arch = "arm")]
pub fn print_stack_pointer() {
    let stack_pointer;
    unsafe { asm!("mov $0, sp" : "=r"(stack_pointer) : : : "volatile") };

    let mut buffer = [b'\n'; 15];
    buffer[0..4].clone_from_slice(b"SP: ");
    write_as_hex(&mut buffer[4..15], stack_pointer);
    Console::new().write(buffer);
}

#[cfg(target_arch = "riscv32")]
pub fn print_stack_pointer() {}

#[inline(always)] // Initial stack size is too small (128 bytes currently)
pub fn dump_address(address: *const usize) {
    let mut buffer = [b' '; 28];
    write_as_hex(&mut buffer[0..10], address as usize);
    buffer[10] = b':';
    write_as_hex(&mut buffer[12..22], unsafe { *address });
    for index in 0..4 {
        let byte = unsafe { *(address as *const u8).offset(index) };
        let byte_is_printable_char = byte >= 0x20 && byte < 0x80;
        if byte_is_printable_char {
            buffer[23 + index as usize] = byte;
        }
    }
    buffer[27] = b'\n';
    Console::new().write(&buffer);
}

pub fn dump_memory(start_address: *const usize, count: isize) {
    let range = if count < 0 { count..0 } else { 0..count };

    for offset in range {
        dump_address(unsafe { start_address.offset(offset) });
    }
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
