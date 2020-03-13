use crate::syscalls;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::ptr;
use core::ptr::NonNull;
use linked_list_allocator::Heap;

pub static mut HEAP: Heap = Heap::empty();

struct TockAllocator;

unsafe impl GlobalAlloc for TockAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HEAP.allocate_first_fit(layout)
            .ok()
            .map_or(ptr::null_mut(), NonNull::as_ptr)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

#[global_allocator]
static ALLOCATOR: TockAllocator = TockAllocator;

#[alloc_error_handler]
unsafe fn alloc_error_handler(_: Layout) -> ! {
    loop {
        syscalls::raw::yieldk();
    }
}
