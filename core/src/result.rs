#[derive(Copy, Clone)]
pub struct SubscribeError {
    pub driver_number: usize,
    pub subscribe_number: usize,
    pub return_code: isize,
}

#[derive(Copy, Clone)]
pub struct CommandError {
    pub driver_number: usize,
    pub command_number: usize,
    pub arg1: usize,
    pub arg2: usize,
    pub return_code: isize,
}

#[derive(Copy, Clone)]
pub struct AllowError {
    pub driver_number: usize,
    pub allow_number: usize,
    pub return_code: isize,
}

pub const SUCCESS: isize = 0;
pub const FAIL: isize = -1;
pub const EBUSY: isize = -2;
pub const EALREADY: isize = -3;
pub const EOFF: isize = -4;
pub const ERESERVE: isize = -5;
pub const EINVAL: isize = -6;
pub const ESIZE: isize = -7;
pub const ECANCEL: isize = -8;
pub const ENOMEM: isize = -9;
pub const ENOSUPPORT: isize = -10;
pub const ENODEVICE: isize = -11;
pub const EUNINSTALLED: isize = -12;
pub const ENOACK: isize = -13;
