// TODO: Implement `libtock_runtime` and `libtock_unittest`, which are
// referenced in the comment on `Syscalls`.

use crate::raw_syscalls::{RawSyscalls, YieldType};

/// `Syscalls` provides safe abstractions over Tock's system calls. It is
/// implemented for `libtock_runtime::TockSyscalls` and
/// `libtock_unittest::FakeSyscalls` (by way of `RawSyscalls`).
pub trait Syscalls {
    /// Puts the process to sleep until a callback becomes pending, invokes the
    /// callback, then returns.
    fn yield_wait();

    /// Runs the next pending callback, if a callback is pending. Unlike
    /// `yield_wait`, `yield_no_wait` returns immediately if no callback is
    /// pending. Returns true if a callback was executed, false otherwise.
    fn yield_no_wait() -> bool;

    // TODO: Add a subscribe interface.

    // TODO: Add a command interface.

    // TODO: Add a read-write allow interface.

    // TODO: Add a read-only allow interface.

    // TODO: Add memop() methods.
}

impl<S: RawSyscalls> Syscalls for S {
    fn yield_wait() {
        Self::raw_yield(YieldType::Wait);
    }

    fn yield_no_wait() -> bool {
        Self::raw_yield(YieldType::NoWait) != ReturnType::Failure as usize
    }
}

// Note: variants are commented out because if they aren't commented out I get a
// "variant is never constructed" error. When we figure out an error handling
// design, this type is likely to move into an error handling-related module, at
// which point we will uncomment the other variants.
enum ReturnType {
    Failure = 0,
    //FailureWithU32 = 1,
    //FailureWith2U32 = 2,
    //FailureWithU64 = 3,
    //Success = 128,
    //SuccessWithU32 = 129,
    //SuccessWith2U32 = 130,
    //SuccessWithU64 = 131,
    //SuccessWith3U32 = 132,
    //SuccessWithU32AndU64 = 133,
}
