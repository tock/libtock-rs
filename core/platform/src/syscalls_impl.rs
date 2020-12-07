//! Implements `Syscalls` for all types that implement `RawSyscalls`.

use crate::{RawSyscalls, Syscalls, YieldType};

impl<S: RawSyscalls> Syscalls for S {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------

    fn yield_wait() {
        Self::raw_yield(YieldType::Wait);
    }

    fn yield_no_wait() -> bool {
        // TODO: Introduce a return type abstraction so this 0 isn't hardcoded.
        Self::raw_yield(YieldType::NoWait) != 0
    }
}
