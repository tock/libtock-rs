//! Implements `Syscalls` for all types that implement `RawSyscalls`.

impl<S: crate::RawSyscalls> crate::Syscalls for S {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------

    fn yield_wait() {
        Self::raw_yield_wait();
    }

    fn yield_no_wait() -> bool {
        let mut flag = 0;
        Self::raw_yield_no_wait(&mut flag);
        flag != 0
    }
}
