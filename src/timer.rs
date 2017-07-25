use core::mem;
use core::cell::Cell;
use syscalls::{self, command, yieldk_for};

pub unsafe fn subscribe(cb: extern fn(usize, usize, usize, usize), ud: usize) {
    syscalls::subscribe(3, 0, cb, ud);
}

pub fn start_oneshot(ms: u32) {
    unsafe {
        command(3, 1, ms as isize);
    }
}

pub fn start_repeating(ms: u32) {
    unsafe {
        command(3, 2, ms as isize);
    }
}

pub fn stop(ms: u32) {
    unsafe {
        command(3, 3, ms as isize);
    }
}

pub fn delay_ms(ms: u32) {
    extern fn cb(_: usize, _: usize, _: usize, expired_ptr: usize) {
        let expired: &Cell<bool> = unsafe {
            mem::transmute(expired_ptr)
        };
        expired.set(true);
    }

    let expired = Cell::new(false);
    unsafe {
        subscribe(cb, &expired as *const _ as usize);
        start_oneshot(ms);
        yieldk_for(|| expired.get() );
    }
}

