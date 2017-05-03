use syscalls::command;

pub fn count() -> isize {
    command(8, 0, 0)
}

pub fn on(led_num: u32) {
    command(8, 1, led_num as isize);
}

pub fn off(led_num: u32) {
    command(8, 2, led_num as isize);
}

pub fn toggle(led_num: u32) {
    command(8, 3, led_num as isize);
}

