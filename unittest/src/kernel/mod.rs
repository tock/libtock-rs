use crate::kernel_data::{with_kernel_data, DriverData, KernelData, KERNEL_DATA};
use crate::{ExpectedSyscall, SyscallLogEntry};

// TODO: Create a fake/ directory, and move this code into fake/kernel.rs.
// Create fake/syscalls/ and move the fake system call implementation into
// fake/syscalls/. Move the system call implementation from `fake::Kernel` to a
// new `fake::Syscalls` type.
// See https://github.com/tock/libtock-rs/issues/313 for a better explanation.

// TODO: Add Allow.

mod command_impl;

#[cfg(test)]
mod command_impl_tests;

// TODO: Add Exit.
// TODO: Add Memop.
// TODO: Add Subscribe.
mod raw_syscalls_impl;
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
// Note: The kernel's data is actually stored in
// crate::kernel_data::KERNEL_DATA. See the kernel_data module comment for an
// explanation.
pub struct Kernel {
    // Prevents user code from constructing a Kernel directly, in order to force
    // construction via new().
    _private: (),
}

impl Kernel {
    /// Creates a `Kernel` for this thread and returns it. The returned `Kernel`
    /// should be dropped at the end of the test, before this thread creates
    /// another `Kernel`.
    // Clippy suggests we implement Default instead of having new. However,
    // using Default implies the newly-created instance isn't special, but it is
    // (as only one can exist per thread). Also, Default::default is not
    // #[track_caller].
    #[allow(clippy::new_without_default)]
    #[track_caller]
    pub fn new() -> Kernel {
        let old_option = KERNEL_DATA.with(|kernel_data| {
            kernel_data.replace(Some(KernelData {
                create_location: std::panic::Location::caller(),
                drivers: Default::default(),
                expected_syscalls: Default::default(),
                syscall_log: Vec::new(),
                upcall_queue: Default::default(),
            }))
        });
        if let Some(old_kernel_data) = old_option {
            panic!(
                "New fake::Kernel created before the previous fake::Kernel \
                 was dropped. The previous fake::Kernel was created at {}.",
                old_kernel_data.create_location
            );
        }
        Kernel { _private: () }
    }

    /// Adds a `fake::Driver` to this `fake::Kernel`. After the call, system
    /// calls with this driver's ID will be routed to the driver.
    // TODO: It's kind of weird to implicitly clone the RC by default. Instead,
    // we should probably take the Rc by value. Also, after making that change,
    // maybe we can take a Rc<dyn fake::Driver> instead of using generics?
    // TODO: Add a test for add_driver.
    pub fn add_driver<D: crate::fake::Driver>(&self, driver: &std::rc::Rc<D>) {
        let id = driver.id();
        let num_upcalls = driver.num_upcalls();
        let driver_data = DriverData {
            driver: driver.clone(),
            num_upcalls,
            upcalls: std::collections::HashMap::with_capacity(num_upcalls as usize),
        };
        let insert_return =
            with_kernel_data(|kernel_data| kernel_data.unwrap().drivers.insert(id, driver_data));
        assert!(insert_return.is_none(), "Duplicate driver with ID {}", id);
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
        with_kernel_data(|kernel_data| {
            kernel_data
                .unwrap()
                .expected_syscalls
                .push_back(expected_syscall)
        });
    }

    /// Returns the system call log and empties it.
    pub fn take_syscall_log(&self) -> Vec<SyscallLogEntry> {
        with_kernel_data(|kernel_data| {
            std::mem::replace(&mut kernel_data.unwrap().syscall_log, Vec::new())
        })
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        KERNEL_DATA.with(|kernel_data| kernel_data.replace(None));
    }
}

// -----------------------------------------------------------------------------
// Crate implementation details below.
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_syscall_queue() {
        use libtock_platform::YieldNoWaitReturn::Upcall;
        use std::matches;
        use ExpectedSyscall::YieldNoWait;
        let kernel = Kernel::new();
        with_kernel_data(|kernel_data| assert!(kernel_data.unwrap().expected_syscalls.is_empty()));
        kernel.add_expected_syscall(YieldNoWait {
            override_return: None,
        });
        kernel.add_expected_syscall(YieldNoWait {
            override_return: Some(Upcall),
        });
        with_kernel_data(|kernel_data| {
            let expected_syscalls = &mut kernel_data.unwrap().expected_syscalls;
            assert!(matches!(
                expected_syscalls.pop_front(),
                Some(YieldNoWait {
                    override_return: None
                })
            ));
            assert!(matches!(
                expected_syscalls.pop_front(),
                Some(YieldNoWait {
                    override_return: Some(Upcall)
                })
            ));
            assert!(expected_syscalls.is_empty());
        });
    }

    #[test]
    fn syscall_log() {
        use SyscallLogEntry::{YieldNoWait, YieldWait};
        let kernel = Kernel::new();
        assert_eq!(kernel.take_syscall_log(), []);
        with_kernel_data(|kernel_data| {
            let syscall_log = &mut kernel_data.unwrap().syscall_log;
            syscall_log.push(YieldNoWait);
            syscall_log.push(YieldWait);
        });
        assert_eq!(kernel.take_syscall_log(), [YieldNoWait, YieldWait]);
        assert_eq!(kernel.take_syscall_log(), []);
    }
}
