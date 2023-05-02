#![forbid(unsafe_code)]
#![no_std]

extern crate libtock_debug_panic;

pub use libtock_platform as platform;
pub use libtock_runtime as runtime;

pub mod alarm {
    use libtock_alarm as alarm;
    pub type Alarm = alarm::Alarm<super::runtime::TockSyscalls>;
    pub use alarm::{Convert, Hz, Milliseconds, Ticks};
}
pub mod ambient_light {
    use libtock_ambient_light as ambient_light;
    pub type AmbientLight = ambient_light::AmbientLight<super::runtime::TockSyscalls>;
}
pub mod buttons {
    use libtock_buttons as buttons;
    pub type Buttons = buttons::Buttons<super::runtime::TockSyscalls>;
    pub use buttons::{ButtonListener, ButtonState};
}
pub mod console {
    use libtock_console as console;
    pub type Console = console::Console<super::runtime::TockSyscalls>;
    pub use console::ConsoleWriter;
}
pub mod gpio {
    use libtock_gpio as gpio;
    pub type Gpio = gpio::Gpio<super::runtime::TockSyscalls>;
    pub use gpio::{
        Error, GpioInterruptListener, GpioState, InputPin, OutputPin, PinInterruptEdge, Pull,
        PullDown, PullNone, PullUp,
    };
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
pub mod proximity {
    use libtock_proximity as proximity;
    pub type Proximity = proximity::Proximity<super::runtime::TockSyscalls>;
}
pub mod temperature {
    use libtock_temperature as temperature;
    pub type Temperature = temperature::Temperature<super::runtime::TockSyscalls>;
    pub use temperature::TemperatureListener;
}
