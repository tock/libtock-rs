
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

pub fn subscribe(major: u32, minor: u32, cb: extern fn(usize, usize, usize, usize), ud: usize) -> u32 {
    let res;
    unsafe {
        asm!("svc 1" : "={r0}"(res)
                     : "{r0}"(major) "{r1}"(minor) "{r2}"(cb) "{r3}"(ud)
                     : "memory"
                     : "volatile");
    }
    res
}

pub fn command(major: u32, minor: u32, arg1: isize) -> isize {
    let res;
    unsafe {
        asm!("svc 2" : "={r0}"(res)
                     : "{r0}"(major) "{r1}"(minor) "{r2}"(arg1)
                     : "memory"
                     : "volatile");
    }
    res
}

