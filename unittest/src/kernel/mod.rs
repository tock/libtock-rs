use crate::{ExpectedSyscall, SyscallLogEntry};
use std::cell::Cell;

// TODO: Add Allow.
// TODO: Add Command.
// TODO: Add Exit.
// TODO: Add Memop.
// TODO: Add Subscribe.
mod raw_syscalls_impl;
mod thread_local;
mod yield_impl;

#[cfg(test)]
mod yield_impl_tests;

/// A fake implementation of the Tock kernel. Provides
/// `libtock_platform::Syscalls` by implementing
/// `libtock_platform::RawSyscalls`. Allows `fake::Driver`s to be attached, and
/// routes system calls to the correct fake driver.
///
/// Note that there can only be one `Kernel` instance per thread, as a
/// thread-local variable is used to implement `libtock_platform::RawSyscalls`.
/// As such, test code is given a `Rc<Kernel>` rather than a `Kernel` instance
/// directly. Because `Rc` is a shared reference, Kernel extensively uses
/// internal mutability.
// TODO: Define the `fake::Driver` trait and add support for fake drivers in
// Kernel.
pub struct Kernel {
    expected_syscalls: Cell<std::collections::VecDeque<ExpectedSyscall>>,

    // The location of the call to `new`. Used by report_leaked() to tell the
    // user which kernel they leaked in a unit test.
    new_location: &'static std::panic::Location<'static>,

    syscall_log: Cell<Vec<SyscallLogEntry>>,
}

impl Kernel {
    /// Creates a `Kernel` for this thread and returns a reference to it. This
    /// instance should be dropped at the end of the test, before this thread
    /// creates another `Kernel`.
    #[track_caller]
    pub fn new() -> std::rc::Rc<Kernel> {
        let rc = std::rc::Rc::new(Kernel {
            expected_syscalls: Default::default(),
            new_location: std::panic::Location::caller(),
            syscall_log: Default::default(),
        });
        thread_local::set_kernel(&rc);
        rc
    }

    /// Adds an ExpectedSyscall to the expected syscall queue.
    ///
    /// # What is the expected syscall queue?
    ///
    /// In addition to routing system calls to drivers, `Kernel` supports
    /// injecting artificial system call responses. The primary use case for
    /// this feature is to simulate errors without having to implement error
    /// simulation in each `fake::Driver`.
    ///
    /// The expected syscall queue is a FIFO queue containing anticipated
    /// upcoming system calls. It starts empty, and as long as it is empty, the
    /// expected syscall functionality does nothing. When the queue is nonempty
    /// and a system call is made, the system call is compared with the next
    /// queue entry. If the system call matches, then the action defined by the
    /// expected syscall is taken. If the call does not match, the call panics
    /// (to make the unit test fail).
    pub fn add_expected_syscall(&self, expected_syscall: ExpectedSyscall) {
        let mut queue = self.expected_syscalls.take();
        queue.push_back(expected_syscall);
        self.expected_syscalls.set(queue);
    }

    /// Returns the system call log and empties it.
    pub fn take_syscall_log(&self) -> Vec<SyscallLogEntry> {
        self.syscall_log.take()
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        thread_local::clear_kernel();
    }
}

// -----------------------------------------------------------------------------
// Crate implementation details below.
// -----------------------------------------------------------------------------

impl Kernel {
    // Appends a log entry to the system call queue.
    fn log_syscall(&self, syscall: SyscallLogEntry) {
        let mut log = self.syscall_log.take();
        log.push(syscall);
        self.syscall_log.set(log);
    }

    // Retrieves the first syscall in the expected syscalls queue, removing it
    // from the queue. Returns None if the queue was empty.
    fn pop_expected_syscall(&self) -> Option<ExpectedSyscall> {
        let mut queue = self.expected_syscalls.take();
        let expected_syscall = queue.pop_front();
        self.expected_syscalls.set(queue);
        expected_syscall
    }

    // Panics, indicating that this Kernel was leaked. It is unlikely that this
    // panic will cause the correct test case to fail, but if this Kernel is
    // well-named the panic message should indicate where the leak occurred.
    fn report_leaked(&self) -> ! {
        panic!(
            "The fake::Kernel initialized at {}:{}:{} was not cleaned up; \
             perhaps a Rc<Kernel> was leaked?",
            self.new_location.file(),
            self.new_location.line(),
            self.new_location.column()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_syscall_queue() {
        use libtock_platform::YieldNoWaitReturn::Upcall;
        use ExpectedSyscall::{YieldNoWait, YieldWait};
        let kernel = Kernel::new();
        assert_eq!(kernel.pop_expected_syscall(), None);
        kernel.add_expected_syscall(YieldNoWait {
            override_return: None,
        });
        kernel.add_expected_syscall(YieldNoWait {
            override_return: Some(Upcall),
        });
        assert_eq!(
            kernel.pop_expected_syscall(),
            Some(YieldNoWait {
                override_return: None
            })
        );
        kernel.add_expected_syscall(YieldWait { skip_upcall: false });
        assert_eq!(
            kernel.pop_expected_syscall(),
            Some(YieldNoWait {
                override_return: Some(Upcall)
            })
        );
        assert_eq!(
            kernel.pop_expected_syscall(),
            Some(YieldWait { skip_upcall: false })
        );
        assert_eq!(kernel.pop_expected_syscall(), None);
    }

    #[test]
    fn syscall_log() {
        use SyscallLogEntry::{YieldNoWait, YieldWait};
        let kernel = Kernel::new();
        assert_eq!(kernel.take_syscall_log(), []);
        kernel.log_syscall(YieldNoWait);
        kernel.log_syscall(YieldWait);
        assert_eq!(kernel.take_syscall_log(), [YieldNoWait, YieldWait]);
        kernel.log_syscall(YieldNoWait);
        assert_eq!(kernel.take_syscall_log(), [YieldNoWait]);
        assert_eq!(kernel.take_syscall_log(), []);
    }

    // Verifies the location propagates correctly into the report_leaked() error
    // message.
    #[test]
    fn name_to_report_leaked() {
        use std::panic::{catch_unwind, AssertUnwindSafe, Location};
        #[rustfmt::skip]
        let (kernel, new_location) = (Kernel::new(), Location::caller());
        let result = catch_unwind(AssertUnwindSafe(|| kernel.report_leaked()));
        let panic_arg = result.expect_err("Kernel::report_leaked did not panic");
        let message = panic_arg
            .downcast_ref::<String>()
            .expect("Wrong panic payload type");
        assert!(message.contains(&format!("{}:{}", new_location.file(), new_location.line())));
    }
}
