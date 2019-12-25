#![feature(asm, alloc_error_handler, lang_items, naked_functions)]
#![cfg_attr(any(target_arch = "arm", target_arch = "riscv32"), no_std)]

mod callback;

pub mod adc;
pub mod ble_composer;
pub mod ble_parser;
pub mod buttons;
pub mod console;
pub mod debug;
pub mod electronics;
pub mod futures;
pub mod gpio;
pub mod led;
pub mod memop;
pub mod result;
pub mod rng;
pub mod sensors;
pub mod shared_memory;
pub mod simple_ble;
pub mod temperature;
pub mod timer;
pub mod unwind_symbols;

#[cfg(any(target_arch = "arm", target_arch = "riscv32"))]
pub mod entry_point;

#[cfg(any(target_arch = "arm", target_arch = "riscv32"))]
mod lang_items;

pub mod syscalls;

use crate::console::ConsoleDriver;
pub use libtock_codegen::main;

/// Dummy structure to force importing the panic_handler and other no_std elements when nothing else
/// is imported.
pub struct LibTock;

/// Struct containing all drivers constructible through retrieve_hardware()
pub struct Hardware {
    pub console_driver: ConsoleDriver,
}

use result::OtherError;
use result::TockError;
use result::TockResult;

/// Retrieve Hardware struct. Returns Hardware only once.
pub fn retrieve_hardware() -> TockResult<Hardware> {
    match unsafe { HARDWARE.take() } {
        Some(hardware) => Ok(hardware),
        None => Err(TockError::Other(OtherError::DriverAlreadyTaken)),
    }
}

static mut HARDWARE: Option<Hardware> = Some(Hardware {
    console_driver: ConsoleDriver {
        _unconstructible: (),
    },
});

#[cfg(test)]
mod test {
    use crate::console::ConsoleDriver;
    use crate::retrieve_hardware;
    use crate::Hardware;
    #[test]
    pub fn can_be_retrieved_once() {
        reset_hardware_singleton();

        assert!(retrieve_hardware().is_ok());
        assert!(retrieve_hardware().is_err());
    }

    fn reset_hardware_singleton() {
        unsafe {
            super::HARDWARE = Some(Hardware {
                console_driver: ConsoleDriver {
                    _unconstructible: (),
                },
            })
        };
    }
}
