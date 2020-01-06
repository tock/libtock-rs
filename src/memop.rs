// Last updated for Tock 1.4.
// See https://github.com/tock/tock/blob/master/doc/syscalls/memop.md
use crate::syscalls;
use core::slice;

/// Set the memory break
/// # Safety
/// Setting the break is marked as unsafe as it should only be called by the entry point to setup
/// the allocator. Updating the break afterwards can lead to allocated memory becoming unaccessible.
///
/// Alternate allocator implementations may still find this useful in the future.
pub unsafe fn set_brk(ptr: *const u8) -> bool {
    syscalls::raw::memop(0, ptr as usize) == 0
}

/// Increment the memory break
pub fn increment_brk(increment: usize) -> Option<*const u8> {
    let result = unsafe { syscalls::raw::memop(1, increment) };
    if result >= 0 {
        Some(result as *const u8)
    } else {
        None
    }
}

pub fn get_brk() -> *const u8 {
    unsafe { syscalls::raw::memop(1, 0) as *const u8 }
}

pub fn get_mem_start() -> *const u8 {
    unsafe { syscalls::raw::memop(2, 0) as *const u8 }
}

pub fn get_mem_end() -> *const u8 {
    unsafe { syscalls::raw::memop(3, 0) as *const u8 }
}

pub fn get_flash_start() -> *const u8 {
    unsafe { syscalls::raw::memop(4, 0) as *const u8 }
}

pub fn get_flash_end() -> *const u8 {
    unsafe { syscalls::raw::memop(5, 0) as *const u8 }
}

pub fn get_grant_start() -> *const u8 {
    unsafe { syscalls::raw::memop(6, 0) as *const u8 }
}

pub fn get_flash_regions_count() -> usize {
    unsafe { syscalls::raw::memop(7, 0) as usize }
}

pub fn get_flash_region_start(i: usize) -> Option<*const u8> {
    if i < get_flash_regions_count() {
        Some(unsafe { syscalls::raw::memop(8, i) as *const u8 })
    } else {
        None
    }
}

pub fn get_flash_region_end(i: usize) -> Option<*const u8> {
    if i < get_flash_regions_count() {
        Some(unsafe { syscalls::raw::memop(9, i) as *const u8 })
    } else {
        None
    }
}

/// It is safe to return a 'static immutable slice, as the Tock kernel doesn't change the layout of
/// flash regions during the application's lifetime.
pub fn get_flash_region(i: usize) -> Option<&'static [u8]> {
    if i < get_flash_regions_count() {
        let start_addr = unsafe { syscalls::raw::memop(8, i) } as usize;
        let start_ptr = start_addr as *const u8;
        let end_addr = unsafe { syscalls::raw::memop(9, i) } as usize;
        // This assumes that the kernel sends consistent results, i.e. start <= end.
        let len = end_addr - start_addr;
        Some(unsafe { slice::from_raw_parts(start_ptr, len) })
    } else {
        None
    }
}

/// Set the top of the stack
/// # Safety
/// Setting the stack_top and heap_start addresses are marked as unsafe as they should only be called
/// by the entry point to setup the allocator. Updating these values afterwards can lead to incorrect
/// debug output from the kernel.
///
/// Alternate allocator implementations may still find this useful in the future.
pub unsafe fn set_stack_top(ptr: *const u8) {
    let _ = syscalls::raw::memop(10, ptr as usize);
}

/// Set the top of the heap
/// # Safety
/// Setting the stack_top and heap_start addresses are marked as unsafe as they should only be called
/// by the entry point to setup the allocator. Updating these values afterwards can lead to incorrect
/// debug output from the kernel.
pub unsafe fn set_heap_start(ptr: *const u8) {
    let _ = syscalls::raw::memop(11, ptr as usize);
}
