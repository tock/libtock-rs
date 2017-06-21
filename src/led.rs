use syscalls::command;

const COUNT: u32 = 0;
const ON: u32 = 1;
const OFF: u32 = 2;
const TOGGLE: u32 = 3;

pub fn count() -> isize {
    command(8, COUNT, 0)
}

pub fn on(led_num: u32) {
    command(8, ON, led_num as isize);
}

pub fn off(led_num: u32) {
    command(8, OFF, led_num as isize);
}

pub fn toggle(led_num: u32) {
    command(8, TOGGLE, led_num as isize);
}
