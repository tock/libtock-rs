
pub fn yieldk() {
    unsafe {
        asm!("push {lr}\n svc 0\n pop {lr}" : : : "memory", "lr" : "volatile");
    }
}

pub fn yieldk_for<F: Fn() -> bool>(cond: F) {
    while !cond() {
        yieldk();
    }
}

pub unsafe fn allow(major: u32, minor: u32, slice: &[u8]) -> isize {
    let res;
    asm!("svc 3" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(slice.as_ptr()) "{r3}"(slice.len())
                 : "memory"
                 : "volatile");
    res
}

pub unsafe fn subscribe(major: u32, minor: u32, cb: extern fn(usize, usize, usize, usize), ud: usize) -> isize {
    let res;
    asm!("svc 1" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(cb) "{r3}"(ud)
                 : "memory"
                 : "volatile");
    res
}

pub unsafe fn command(major: u32, minor: u32, arg1: isize) -> isize {
    let res;
    asm!("svc 2" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(arg1)
                 : "memory"
                 : "volatile");
    res
}

pub unsafe fn memop(major: u32, arg1: usize) -> isize {
    let res;
    asm!("svc 4" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(arg1)
                 : "memory"
                 : "volatile");
    res
}

