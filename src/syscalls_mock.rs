pub fn yieldk_for<F: Fn() -> bool>(_: F) {
    unimplemented()
}

pub unsafe fn allow(_: usize, _: usize, _: &[u8]) -> isize {
    unimplemented()
}

pub unsafe fn allow16(_: usize, _: usize, _: &[u16]) -> isize {
    unimplemented()
}

pub unsafe fn subscribe(
    _: usize,
    _: usize,
    _: unsafe extern "C" fn(usize, usize, usize, usize),
    _: usize,
) -> isize {
    unimplemented()
}

pub unsafe fn subscribe_signed(
    _: usize,
    _: usize,
    _: unsafe extern "C" fn(isize, usize, usize, usize),
    _: usize,
) -> isize {
    unimplemented()
}

pub fn unsubscribe(_: usize, _: usize) -> isize {
    unimplemented()
}

pub unsafe fn command(_: usize, _: usize, _: usize, _: usize) -> isize {
    unimplemented()
}

fn unimplemented() -> ! {
    unimplemented!("Unimplemented for tests");
}
