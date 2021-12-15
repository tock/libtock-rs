#![no_std]

use libtock_low_level_debug::{AlertCode, LowLevelDebug};
use libtock_platform::{ErrorCode, Syscalls};
use libtock_runtime::TockSyscalls;

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    // Signal a panic using the LowLevelDebug capsule (if available).
    LowLevelDebug::<TockSyscalls>::print_alert_code(AlertCode::Panic);

    // Exit with a non-zero exit code to indicate failure.
    // TODO(kupiakos@google.com): Make this logic consistent with tock/tock#2914
    // when it is merged.
    TockSyscalls::exit_terminate(ErrorCode::Fail as u32);
}
