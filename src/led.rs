use syscalls::command;

const COUNT: u32 = 0;
const ON: u32 = 1;
const OFF: u32 = 2;
const TOGGLE: u32 = 3;
const LED_DRIVER: u32 = 2;

pub fn count() -> isize {
    unsafe {
        command(LED_DRIVER, COUNT, 0)
    }
}

pub fn on(led_num: u32) {
    unsafe {
        command(LED_DRIVER, ON, led_num as isize);
    }
}

pub fn off(led_num: u32) {
    unsafe {
        command(LED_DRIVER, OFF, led_num as isize);
    }
}

pub fn toggle(led_num: u32) {
    unsafe {
        command(LED_DRIVER, TOGGLE, led_num as isize);
    }
}
