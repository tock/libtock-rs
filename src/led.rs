use syscalls::command;

const COUNT: u32 = 0;
const ON: u32 = 1;
const OFF: u32 = 2;
const TOGGLE: u32 = 3;
const DRIVER_NUMBER: u32 = 0x00002;

pub fn count() -> isize {
    unsafe { command(DRIVER_NUMBER, COUNT, 0) }
}

pub fn on(led_num: u32) {
    unsafe {
        command(DRIVER_NUMBER, ON, led_num as isize);
    }
}

pub fn off(led_num: u32) {
    unsafe {
        command(DRIVER_NUMBER, OFF, led_num as isize);
    }
}

pub fn toggle(led_num: u32) {
    unsafe {
        command(DRIVER_NUMBER, TOGGLE, led_num as isize);
    }
}
