/// Unit tests can use `ExpectedSyscall` to alter `fake::Kernel`'s behavior for
/// a particular system call. An example use case is error injection: unit tests
/// can add a `ExpectedSyscall` to the fake kernel's queue to insert errors in
/// order to test error handling code.
#[derive(Debug, PartialEq)]
pub enum ExpectedSyscall {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------
    YieldNoWait {
        /// If not `None`, `yield-no-wait` will set the return value to the
        /// specified value. If `None`, `yield-no-wait` will set the return
        /// value based on whether or not an upcall was run.
        override_return: Option<libtock_platform::YieldNoWaitReturn>,
    },

    YieldWait {
        /// If true, yield_wait will skip executing a upcall.
        skip_upcall: bool,
    },
    // TODO: Add Subscribe.
    // TODO: Add Command.
    // TODO: Add Allow.
    // TODO: Add Memop.
    // TODO: Add Exit.
}

impl ExpectedSyscall {
    // Panics with a message describing that the named system call was called
    // instead of the expected system call. Used by fake::Kernel to report
    // incorrect system calls.
    pub(crate) fn panic_wrong_call(&self, called: &str) -> ! {
        // TODO: Implement Display for ExpectedSyscall and replace {:?} with {}
        panic!(
            "Expected system call {:?}, but {} was called instead.",
            self, called
        );
    }
}
