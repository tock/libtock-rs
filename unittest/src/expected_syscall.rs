/// Unit tests can use `ExpectedSyscall` to alter `fake::Kernel`'s behavior for
/// a particular system call. An example use case is error injection: unit tests
/// can add a `ExpectedSyscall` to the fake kernel's queue to insert errors in
/// order to test error handling code.
#[derive(Debug)]
pub enum ExpectedSyscall {
    // TODO: Add Yield.
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
    #[allow(unused)] // TODO: Remove when a system call is implemented.
    pub(crate) fn panic_wrong_call(&self, called: &str) -> ! {
        // TODO: Implement Display for ExpectedSyscall and replace {:?} with {}
        panic!(
            "Expected system call {:?}, but {} was called instead.",
            self, called
        );
    }
}
