#![no_std]

use libtock2::{
    low_level_debug::{AlertCode, LowLevelDebug},
    platform::Syscalls,
    runtime::TockSyscalls,
};

#[panic_handler]
unsafe fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    report_panic()
}

unsafe fn report_panic() -> ! {
    // Signal a panic using the LowLevelDebug capsule (if available).
    LowLevelDebug::print_alert_code(AlertCode::Panic);

    loop {
        TockSyscalls::yield_wait();
    }
}
