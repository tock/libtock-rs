pub fn yieldk() {}

pub unsafe fn subscribe(
    _: usize,
    _: usize,
    _: *const unsafe extern "C" fn(usize, usize, usize, usize),
    _: usize,
) -> isize {
    unimplemented()
}

pub unsafe fn command(_: usize, _: usize, _: usize, _: usize) -> isize {
    unimplemented()
}

pub unsafe fn command1(_: usize, _: usize, _: usize) -> isize {
    unimplemented()
}

pub unsafe fn allow(_: usize, _: usize, _: *mut u8, _: usize) -> isize {
    unimplemented()
}

pub unsafe fn memop(_: u32, _: usize) -> isize {
    unimplemented()
}

fn unimplemented() -> ! {
    unimplemented!("Unimplemented for tests");
}
