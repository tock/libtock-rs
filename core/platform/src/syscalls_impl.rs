//! Implements `Syscalls` for all types that implement `RawSyscalls`.

use crate::{RawSyscalls, Syscalls, YieldNoWaitReturn};

mod yield_op {
    pub const NO_WAIT: u32 = 0;
    pub const WAIT: u32 = 1;
}

impl<S: RawSyscalls> Syscalls for S {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------

    fn yield_no_wait() -> YieldNoWaitReturn {
        unsafe {
            // flag can be uninitialized because it is not read before the yield
            // system call, and the kernel promises to only write to it (not
            // read it).
            let mut flag = core::mem::MaybeUninit::<YieldNoWaitReturn>::uninit();

            // flag is safe to write a YieldNoWaitReturn to, as guaranteed by
            // MaybeUninit.
            Self::yield2(yield_op::NO_WAIT as *mut (), flag.as_mut_ptr() as *mut ());

            // yield-no-wait guarantees it sets (initializes) flag before
            // returning.
            flag.assume_init()
        }
    }

    fn yield_wait() {
        // Safety: yield-wait does not return a value, which satisfies yield1's
        // requirement. The yield-wait system call cannot trigger undefined
        // behavior on its own in any other way.
        unsafe {
            Self::yield1(yield_op::WAIT as *mut ());
        }
    }
}
