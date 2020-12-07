//! Implements `Syscalls` for all types that implement `RawSyscalls`.

use crate::{return_type, RawSyscalls, Syscalls, YieldType};

impl<S: RawSyscalls> Syscalls for S {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------

    fn yield_wait() {
        Self::raw_yield(YieldType::Wait);
    }

    fn yield_no_wait() -> bool {
        Self::raw_yield(YieldType::NoWait) != return_type::FAILURE.into()
    }
}
