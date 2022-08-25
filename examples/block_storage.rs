//! A simple block_storage example.
//! Prints device geometry, then assumes its write block size is under 256 bytes,
//! and erases a block.

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::block_storage::BlockStorage;
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x1800}

fn main() {
    let mut w = Console::writer();
    if BlockStorage::driver_check() {
        let g = BlockStorage::get_geometry();
        writeln!(&mut w, "Write block size: {} bytes", g.write_block_size).unwrap();
        writeln!(
            Console::writer(),
            "Erase block size: {} bytes",
            g.erase_block_size
        )
        .unwrap();
        let mut buf = [0; 256];
        if g.write_block_size as usize > buf.len() {
            writeln!(
                Console::writer(),
                "Block size bigger than preallocated buffer, writes will be inaccurate."
            )
            .unwrap();
        }
        let wb = 43;
        let a = g.get_address_of_write_block(wb);
        let eb = g.get_erase_block_containing(a);
        BlockStorage::read(wb, &mut buf).unwrap();
        writeln!(&mut w, "First bytes of sector 43: {:?}", &buf[..10]).unwrap();
        BlockStorage::erase(eb).unwrap();
        BlockStorage::read(wb, &mut buf).unwrap();
        writeln!(&mut w, "Erased sector to: {:?}", &buf[..10]).unwrap();
        buf = [0; 256];
        buf[2] = 137;
        BlockStorage::write(wb, &mut buf).unwrap();
        BlockStorage::read(wb, &mut buf).unwrap();
        writeln!(&mut w, "Written sector: {:?}", &buf[..10]).unwrap();
    } else {
        writeln!(Console::writer(), "No block device detected").unwrap();
    }
}
