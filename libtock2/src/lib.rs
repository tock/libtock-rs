#![forbid(unsafe_code)]
#![no_std]

extern crate libtock_small_panic;

pub use libtock_platform as platform;
pub use libtock_runtime as runtime;

pub mod buttons {
    use libtock_buttons as buttons;
    pub type Buttons = buttons::Buttons<super::runtime::TockSyscalls>;
}
pub mod console {
    use libtock_console as console;
    pub type Console = console::Console<super::runtime::TockSyscalls>;
}
pub mod leds {
    use libtock_leds as leds;
    pub type Leds = leds::Leds<super::runtime::TockSyscalls>;
}
pub mod low_level_debug {
    use libtock_low_level_debug as lldb;
    pub type LowLevelDebug = lldb::LowLevelDebug<super::runtime::TockSyscalls>;
    pub use lldb::AlertCode;
}
