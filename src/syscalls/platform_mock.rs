/// yield for a callback fired by the kernel
/// # Safety
/// Yielding inside a callback conflicts with Rust's safety guarantees. For example,
/// a FnMut closure could be triggered multiple times making a &mut a shared reference.
pub unsafe fn yieldk() {}

/// Subscribe a callback to the kernel
/// # Safety
/// Unsafe as passed callback is dereferenced and called.
pub unsafe fn subscribe(
    _: usize,
    _: usize,
    _: *const unsafe extern "C" fn(usize, usize, usize, usize),
    _: usize,
) -> isize {
    unimplemented()
}

pub fn command(_: usize, _: usize, _: usize, _: usize) -> isize {
    unimplemented()
}

/// Call a command only taking into accoun the first argument
/// # Safety
/// Unsafe as ignored arguments cause leaking of registers to the kernel
pub unsafe fn command1(_: usize, _: usize, _: usize) -> isize {
    unimplemented()
}

/// Share a memory region with the kernel
/// # Safety
/// Unsafe as the pointer to the shared buffer is potentially dereferenced by the kernel.
pub unsafe fn allow(_: usize, _: usize, _: *mut u8, _: usize) -> isize {
    unimplemented()
}

/// Generic operations on the app's memory as requesting more memory
/// # Safety
/// Allows the kernel to do generic operations on the app's memory which can cause memory corruption.
pub unsafe fn memop(_: u32, _: usize) -> isize {
    unimplemented()
}

fn unimplemented() -> ! {
    unimplemented!("Unimplemented for tests");
}
