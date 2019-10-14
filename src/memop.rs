// Last updated for Tock 1.4.
// See https://github.com/tock/tock/blob/master/doc/syscalls/memop.md
use crate::syscalls;
use core::slice;

// Setting the break is marked as unsafe as it should only be called by the entry point to setup
// the allocator. Updating the break afterwards can lead to allocated memory becoming unaccessible.
//
// Alternate allocator implementations may still find this useful in the future.
pub unsafe fn set_brk(ptr: *const u8) -> bool {
    syscalls::memop(0, ptr as usize) == 0
}

pub unsafe fn increment_brk(increment: usize) -> Option<*const u8> {
    let result = syscalls::memop(1, increment);
    if result >= 0 {
        Some(result as *const u8)
    } else {
        None
    }
}

pub fn get_brk() -> *const u8 {
    unsafe { syscalls::memop(1, 0) as *const u8 }
}

pub fn get_mem_start() -> *const u8 {
    unsafe { syscalls::memop(2, 0) as *const u8 }
}

pub fn get_mem_end() -> *const u8 {
    unsafe { syscalls::memop(3, 0) as *const u8 }
}

pub fn get_flash_start() -> *const u8 {
    unsafe { syscalls::memop(4, 0) as *const u8 }
}

pub fn get_flash_end() -> *const u8 {
    unsafe { syscalls::memop(5, 0) as *const u8 }
}

pub fn get_grant_start() -> *const u8 {
    unsafe { syscalls::memop(6, 0) as *const u8 }
}

pub fn get_flash_regions_count() -> usize {
    unsafe { syscalls::memop(7, 0) as usize }
}

pub fn get_flash_region_start(i: usize) -> Option<*const u8> {
    if i < get_flash_regions_count() {
        Some(unsafe { syscalls::memop(8, i) as *const u8 })
    } else {
        None
    }
}

pub fn get_flash_region_end(i: usize) -> Option<*const u8> {
    if i < get_flash_regions_count() {
        Some(unsafe { syscalls::memop(9, i) as *const u8 })
    } else {
        None
    }
}

// It is safe to return a 'static immutable slice, as the Tock kernel doesn't change the layout of
// flash regions during the application's lifetime.
pub fn get_flash_region(i: usize) -> Option<&'static [u8]> {
    if i < get_flash_regions_count() {
        let start = unsafe { syscalls::memop(8, i) as *const u8 };
        let end = unsafe { syscalls::memop(9, i) as *const u8 };
        // This assumes that the kernel sends consistent results, i.e. start <= end.
        let len = unsafe { end.offset_from(start) } as usize;
        Some(unsafe { slice::from_raw_parts(start, len) })
    } else {
        None
    }
}

// Setting the stack_top and heap_start addresses are marked as unsafe as they should only be called
// by the entry point to setup the allocator. Updating these values afterwards can lead to incorrect
// debug output from the kernel.
//
// Alternate allocator implementations may still find this useful in the future.
pub unsafe fn set_stack_top(ptr: *const u8) {
    let _ = syscalls::memop(10, ptr as usize);
}

pub unsafe fn set_heap_start(ptr: *const u8) {
    let _ = syscalls::memop(11, ptr as usize);
}
