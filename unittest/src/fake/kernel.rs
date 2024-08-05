use crate::kernel_data::{with_kernel_data, DriverData, KernelData, KERNEL_DATA};
use crate::{DriverShareRef, ExpectedSyscall, SyscallLogEntry};
use std::cell::Cell;

/// A fake implementation of the Tock kernel. Used with `fake::Syscalls`, which
/// provides system calls that are routed to this kernel. `fake::SyscallDriver`s
/// may be attached to a `fake::Kernel`, and the `fake::Kernel` will route
/// system calls to the correct fake driver.
///
/// Note that there can only be one `fake::Kernel` instance per thread, as
/// `fake::Syscalls` uses a thread-local variable to locate the `fake::Kernel`.
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
                allow_db: Default::default(),
                create_location: std::panic::Location::caller(),
                drivers: Default::default(),
                expected_syscalls: Default::default(),
                syscall_log: Vec::new(),
                upcall_queue: Default::default(),
                memory_break: core::ptr::null(),
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

    /// Adds a `fake::SyscallDriver` to this `fake::Kernel`. After the call,
    /// system calls with this driver's ID will be routed to the driver.
    // TODO: It's kind of weird to implicitly clone the RC by default. Instead,
    // we should probably take the Rc by value. Also, after making that change,
    // maybe we can take a Rc<dyn fake::SyscallDriver> instead of using
    // generics?
    // TODO: Add a test for add_driver.
    pub fn add_driver<D: crate::fake::SyscallDriver>(&self, driver: &std::rc::Rc<D>) {
        let info = driver.info();
        let driver_data = DriverData {
            driver: driver.clone(),
            num_upcalls: info.upcall_count,
            upcalls: std::collections::HashMap::with_capacity(info.upcall_count as usize),
        };
        let insert_return = with_kernel_data(|kernel_data| {
            kernel_data
                .unwrap()
                .drivers
                .insert(info.driver_num, driver_data)
        });
        assert!(
            insert_return.is_none(),
            "Duplicate driver with number {}",
            info.driver_num
        );
        driver.register(DriverShareRef {
            driver_num: Cell::new(info.driver_num),
        });
    }

    /// Adds an ExpectedSyscall to the expected syscall queue.
    ///
    /// # What is the expected syscall queue?
    ///
    /// In addition to routing system calls to drivers, `Kernel` supports
    /// injecting artificial system call responses. The primary use case for
    /// this feature is to simulate errors without having to implement error
    /// simulation in each `fake::SyscallDriver`.
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
        with_kernel_data(|kernel_data| std::mem::take(&mut kernel_data.unwrap().syscall_log))
    }

    /// Returns true if the specified driver installed.
    pub fn is_driver_present(driver_num: u32) -> bool {
        with_kernel_data(|kernel_data| {
            kernel_data.map_or(false, |kernel| kernel.drivers.contains_key(&driver_num))
        })
    }

    /// Returns true if there are any pending upcalls.
    pub fn is_upcall_pending() -> bool {
        with_kernel_data(|kernel_data| {
            kernel_data.map_or(false, |kernel| !kernel.upcall_queue.is_empty())
        })
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        KERNEL_DATA.with(|kernel_data| kernel_data.replace(None));
    }
}
