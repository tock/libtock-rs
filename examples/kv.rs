//! A key-value store example. Gets and sets key-value objects.

#![no_main]
#![no_std]

use core::fmt::Write;
use core::str;
use libtock::console::Console;

use libtock::key_value::KeyValue;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x200}

/// Retrieve a key and insert the value into `value`. Then print the value
/// contents as a UTF-8 string or as hex values.
fn get_and_print(key: &[u8], value: &mut [u8]) {
    match KeyValue::get(key, value) {
        Ok(val_length) => {
            let val_length: usize = val_length as usize;
            writeln!(Console::writer(), "Got value len: {}", val_length).unwrap();

            match str::from_utf8(&value[0..val_length]) {
                Ok(val_str) => writeln!(Console::writer(), "Value: {}", val_str).unwrap(),
                Err(_) => {
                    write!(Console::writer(), "Value: ").unwrap();
                    for i in 0..val_length {
                        write!(Console::writer(), "{:02x}", value[i]).unwrap();
                    }
                    write!(Console::writer(), "\n").unwrap();
                }
            }
        }
        Err(_) => writeln!(Console::writer(), "error KV::get()",).unwrap(),
    }
}

fn main() {
    // Check if there is key-value support on this board.
    if KeyValue::exists() {
        writeln!(Console::writer(), "KV available!").unwrap()
    } else {
        writeln!(Console::writer(), "ERR! KV unavailable").unwrap();
    }

    // Do a test query for key: a.
    let key = "a";
    let mut value: [u8; 64] = [0; 64];
    get_and_print(key.as_bytes(), &mut value);

    // Now set, get, delete, then get the key: libtock-rs.
    let set_key = "libtock-rs";
    let set_val = "kv test app";

    match KeyValue::set(set_key.as_bytes(), set_val.as_bytes()) {
        Ok(()) => {
            writeln!(Console::writer(), "Successfully set the key-value").unwrap();
        }
        Err(_) => writeln!(Console::writer(), "error KV::set()",).unwrap(),
    }

    get_and_print(set_key.as_bytes(), &mut value);

    match KeyValue::delete(set_key.as_bytes()) {
        Ok(()) => {
            writeln!(Console::writer(), "Successfully deleted the key-value").unwrap();
        }
        Err(_) => writeln!(Console::writer(), "error KV::delete()",).unwrap(),
    }

    get_and_print(set_key.as_bytes(), &mut value);
}
