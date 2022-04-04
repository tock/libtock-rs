#![no_std]
use core::fmt::Write;
use libtock_console::Console;
use libtock_low_level_debug::{AlertCode, LowLevelDebug};
use libtock_platform::{ErrorCode, Syscalls};
use libtock_runtime::TockSyscalls;

/// This handler requires some 0x400 bytes of stack

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // Signal a panic using the LowLevelDebug capsule (if available).
    LowLevelDebug::<TockSyscalls>::print_alert_code(AlertCode::Panic);

    let mut writer = Console::<TockSyscalls>::writer();
    // If this printing fails, we can't panic harder, and we can't print it either.
    let _ = writeln!(writer, "{}", info);
    // Exit with a non-zero exit code to indicate failure.
    TockSyscalls::exit_terminate(ErrorCode::Fail as u32);
}
