//! Heapless debugging functions for Tock troubleshooting

mod low_level_debug;

use crate::drivers;
use libtock_core::debug as core_debug;

pub use low_level_debug::*;

pub fn println() {
    let buffer = [b'\n'];
    let drivers = unsafe { drivers::retrieve_drivers_unsafe() };
    let mut console = drivers.console.create_console();
    let _ = console.write(&buffer);
}

pub fn print_as_hex(value: usize) {
    let mut buffer = [b'\n'; 11];
    write_as_hex(&mut buffer, value);
    let drivers = unsafe { drivers::retrieve_drivers_unsafe() };
    let mut console = drivers.console.create_console();
    let _ = console.write(buffer);
}

pub fn print_stack_pointer() {
    let mut buffer = [b'\n'; 15];
    buffer[0..4].clone_from_slice(b"SP: ");
    write_as_hex(&mut buffer[4..15], core_debug::get_stack_pointer());
    let drivers = unsafe { drivers::retrieve_drivers_unsafe() };
    let mut console = drivers.console.create_console();
    let _ = console.write(buffer);
}

#[inline(always)]
/// Dumps address
/// # Safety
/// dereferences pointer without check - may lead to access violation.
pub unsafe fn dump_address(address: *const usize) {
    let mut buffer = [b' '; 28];
    write_as_hex(&mut buffer[0..10], address as usize);
    buffer[10] = b':';
    write_as_hex(&mut buffer[12..22], *address);
    for index in 0..4 {
        let byte = *(address as *const u8).offset(index);
        let byte_is_printable_char = byte >= 0x20 && byte < 0x80;
        if byte_is_printable_char {
            buffer[23 + index as usize] = byte;
        }
    }
    buffer[27] = b'\n';
    let drivers = drivers::retrieve_drivers_unsafe();
    let mut console = drivers.console.create_console();
    let _ = console.write(&buffer);
}

/// Dumps arbitrary memory regions.
/// # Safety
/// dereferences pointer without check - may lead to access violation.
pub unsafe fn dump_memory(start_address: *const usize, count: isize) {
    let range = if count < 0 { count..0 } else { 0..count };

    for offset in range {
        dump_address(start_address.offset(offset));
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
