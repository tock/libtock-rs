use syscalls;

const DRIVER_NUMBER: u32 = 0x00003;

mod command_nr {
    pub const COUNT: u32 = 0;
    pub const ENABLE_INTERRUPT: u32 = 1;
    pub const DISALE_INTERRUPT: u32 = 2;
    pub const READ: u32 = 3;
}

pub struct Button {
    button_num: isize,
}

pub fn count() -> isize {
    unsafe { syscalls::command(DRIVER_NUMBER, command_nr::COUNT, 0) }
}

pub fn get(button_num: isize) -> Option<Button> {
    if button_num >= 0 && button_num < count() {
        Some(Button { button_num })
    } else {
        None
    }
}

pub fn all() -> ButtonIter {
    ButtonIter {
        curr_button: 0,
        button_count: count(),
    }
}

impl Button {
    pub fn read(&self) -> bool {
        unsafe { syscalls::command(DRIVER_NUMBER, command_nr::READ, self.button_num) == 1 }
    }
}

#[derive(Copy, Clone)]
pub struct ButtonIter {
    curr_button: isize,
    button_count: isize,
}

impl Iterator for ButtonIter {
    type Item = Button;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_button < self.button_count {
            let item = Button {
                button_num: self.curr_button,
            };
            self.curr_button += 1;
            Some(item)
        } else {
            None
        }
    }
}
