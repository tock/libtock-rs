/// SyscallLogEntry represents a system call made during test execution.
#[derive(Debug, PartialEq)]
pub enum SyscallLogEntry {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------
    YieldNoWait,

    YieldWait,

    // TODO: Add Subscribe.

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------
    Command {
        driver_id: u32,
        command_id: u32,
        argument0: u32,
        argument1: u32,
    },

    // TODO: Add Allow.
    // TODO: Add Memop.
    // TODO: Add Exit.
}
