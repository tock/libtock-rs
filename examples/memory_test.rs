#![no_std]
use libtock::memop::*;
use core::fmt::Write;
use libtock::result::TockResult;
use libtock::syscalls::raw::*;

#[libtock::main]
async fn main() -> TockResult<()> {
    let drivers = libtock::retrieve_drivers()?;

    let mut console = drivers.console.create_console();

    writeln!(console, "Starting PMP test")?;

    let address = get_mem_start() as usize;
    let address_end = get_mem_end() as usize;

    writeln!(console, "  mem_start/mem_len: {:x}/{:x}", address, address_end)?;

/************* READ TEST MEMORY - PASSING ****************/
    writeln!(console, "")?;
    writeln!(console, "Starting memory read test: Reading from {:x} to {:x}", address, address_end)?;

    for addr in address..address_end {
        let ptr = addr as *mut u32;

        if (addr % 0x200) == 0 {
            writeln!(console, "    reading from: 0x{:x}", addr)?;
        }

        unsafe {
            core::ptr::read_volatile(ptr);
        }
    }

    writeln!(console, "  Finished memory read")?;

/************* READ TEST FLASH - PASSING ****************/
    let flash = get_flash_start() as usize;
    let flash_end = get_flash_end() as usize;

    writeln!(console, "Starting flash read test: Reading from {:x} to {:x}", flash, flash_end)?;

    for addr in flash..flash_end {
        let ptr = addr as *mut u32;

        if (addr % 0x1000) == 0 {
            writeln!(console, "    reading from: 0x{:x}", addr)?;
        }

        unsafe {
            core::ptr::read_volatile(ptr);
        }
    }

    writeln!(console, "  Finished flash read")?;

/************* WRITE TEST MEMORY WITH INC - PASSING ****************/
    let brk_og = get_brk() as usize;
    increment_brk(0x400);
    let brk = get_brk() as usize;

    writeln!(console, "Incremented BRK from: 0x{:x} to 0x{:x}", brk_og, brk)?;

    writeln!(console, "Increment BRK to 0x{:x}", brk)?;

    writeln!(console, "Starting memory inc write test: Writing to 0x{:x} to 0x{:x}", brk_og, brk)?;

    for addr in brk_og..brk {
        let ptr = addr as *mut u32;

        if (addr % 0x100) == 0 {
            writeln!(console, "    writing to: 0x{:x}", addr)?;
        }

        unsafe {
            core::ptr::write_volatile(ptr, 0xDEADBEEF);
        }
    }

    writeln!(console, "  Finished brk inc write")?;

/************* READ TESTS - FAILING ****************/
    // writeln!(console, "")?;
    // writeln!(console, "Starting memory read test: Reading from invalid address {:x} to {:x}", address_end, address_end + 0x100)?;

    // for addr in address_end..(address_end + 0x100) {
    //     let ptr = addr as *mut u32;

    //     writeln!(console, "    reading from: 0x{:x}", addr)?;

    //     unsafe {
    //         core::ptr::read_volatile(ptr);
    //     }
    // }

    // writeln!(console, "  Finished memory read")?;

/************* READ TEST FLASH - FAILING ****************/
    // let flash_end = get_flash_end() as usize;

    // writeln!(console, "Starting flash read test: Reading from invalid address {:x} to {:x}", flash_end, flash_end + 0x100)?;

    // for addr in flash_end..flash_end + 0x100 {
    //     let ptr = addr as *mut u32;

    //     writeln!(console, "    reading from: 0x{:x}", addr)?;

    //     unsafe {
    //         core::ptr::read_volatile(ptr);
    //     }
    // }

    // writeln!(console, "  Finished flash read")?;


    writeln!(console, "Done!")?;

    loop {
        unsafe{ yieldk(); }
    }
}
