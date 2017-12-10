use core::cell::Cell;
use syscalls::{self, command, yieldk_for};

const DRIVER_NUMBER: u32 = 0x00000;
const GET_CLOCK_FREQUENCY: u32 = 1;
const SET_ALARM_NOTIFICATION: u32 = 4;
const GET_CLOCK_VALUE: u32 = 2;


pub unsafe fn subscribe(cb: extern "C" fn(usize, usize, usize, usize), ud: usize) {
    syscalls::subscribe(DRIVER_NUMBER, 0, cb, ud);
}

pub fn set_alarm(ms: u32) {
    unsafe {
        command(DRIVER_NUMBER, SET_ALARM_NOTIFICATION, ms as isize);
    }
}

// FIXME: Broken!
pub fn start_repeating(ms: u32) {
    unsafe {
        command(DRIVER_NUMBER, 2, ms as isize);
    }
}

pub fn stop(ms: u32) {
    unsafe {
        command(DRIVER_NUMBER, 3, ms as isize);
    }
}

pub fn delay_ms(ms: u32) {
    extern "C" fn cb(_: usize, _: usize, _: usize, expired_ptr: usize) {
        let expired = unsafe { &*(expired_ptr as *const Cell<bool>) };
        expired.set(true);
    }

    let expired = Cell::new(false);
    let f: u32 = unsafe { command(DRIVER_NUMBER, GET_CLOCK_FREQUENCY, 0) as u32 };
    let point: u32 = unsafe { command(DRIVER_NUMBER, GET_CLOCK_VALUE, 0) as u32 } + ms * f / 1000;

    unsafe {
        subscribe(cb, &expired as *const _ as usize);
        set_alarm(point);
        yieldk_for(|| expired.get());
    }
}
