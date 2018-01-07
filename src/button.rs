use syscalls;

const DRIVER_NUMBER: u32 = 0x00003;

mod command_nr {
    pub const COUNT: u32 = 0;
    pub const ENABLE_INTERRUPT: u32 = 1;
    pub const DISALE_INTERRUPT: u32 = 2;
    pub const READ: u32 = 3;
    pub const SUBSCRIBE_CALLBACK: u32 = 0;
}

pub struct ButtonUninitialized {
    button_num: isize,
}

pub struct ButtonRead {
    button_num: isize,
}

pub fn count() -> isize {
    unsafe { syscalls::command(DRIVER_NUMBER, command_nr::COUNT, 0) }
}

pub fn get(button_num: isize) -> Option<ButtonUninitialized> {
    if button_num >= 0 && button_num < count() {
        Some(ButtonUninitialized { button_num })
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

impl ButtonUninitialized {
    fn new(button_num: isize) -> ButtonUninitialized {
        ButtonUninitialized { button_num }
    }
    pub fn initialize(
        self,
        callback: Option<extern "C" fn(usize, usize, usize, usize)>,
    ) -> Result<ButtonRead, &'static str> {
        let callback = callback.unwrap_or(noop_callback);
        self.subscribe_callback(callback).and_then(|button| {
            button.enable_callback()
        })
    }

    fn subscribe_callback(
        self,
        callback: extern "C" fn(usize, usize, usize, usize),
    ) -> Result<ButtonUninitialized, &'static str> {
        if unsafe {
            syscalls::subscribe(
                DRIVER_NUMBER,
                command_nr::SUBSCRIBE_CALLBACK,
                callback,
                self.button_num as usize,
            )
        } == 0
        {
            Ok(self)
        } else {
            Err("Could not subscribe callback.")
        }
    }

    fn enable_callback(self) -> Result<ButtonRead, &'static str> {
        if unsafe {
            syscalls::command(DRIVER_NUMBER, command_nr::ENABLE_INTERRUPT, self.button_num)
        } == 0
        {
            Ok(ButtonRead { button_num: self.button_num })
        } else {
            Err("Could not enable callback.")
        }

    }
}
impl ButtonRead {
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
    type Item = ButtonUninitialized;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_button < self.button_count {
            let item = ButtonUninitialized::new(self.curr_button);
            self.curr_button += 1;
            Some(item)
        } else {
            None
        }
    }
}

extern "C" fn noop_callback(_: usize, _: usize, _: usize, ptr: usize) {}
