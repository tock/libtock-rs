/// SyscallLogEntry represents a system call made during test execution.
#[derive(Debug, PartialEq)]
pub enum SyscallLogEntry {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------
    YieldNoWait,

    YieldWait,
    // TODO: Add Subscribe.
    // TODO: Add Command.
    // TODO: Add Allow.
    // TODO: Add Memop.
    // TODO: Add Exit.
}
