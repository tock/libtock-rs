//! `KernelData` contains the data corresponding to a `fake::Kernel`. It is
//! stored in the thread-local variable `KERNEL_DATA`.
//!
//! The data is stored separately from the `fake::Kernel` because in addition to
//! being accessed through the `fake::Kernel`, it is also accessed by
//! `fake::Syscalls` and `upcall::schedule`. `fake::Syscalls` is reentrant (a
//! Yield invocation can run a callback that executes another system call),
//! which easily results in messy code. To keep things understandable, code that
//! uses `KERNEL_DATA` should avoid calling user-supplied functions (such as
//! upcalls) while holding a reference to `KERNEL_DATA`.

use std::cell::RefCell;

pub(crate) struct KernelData {
    pub allow_db: crate::allow_db::AllowDb,

    // The location of the call to `fake::Kernel::new`. Used in the event a
    // duplicate `fake::Kernel` is created to tell the user which kernel they
    // did not clean up in a unit test.
    pub create_location: &'static std::panic::Location<'static>,

    pub drivers: std::collections::HashMap<u32, DriverData>,
    pub expected_syscalls: std::collections::VecDeque<crate::ExpectedSyscall>,
    pub syscall_log: Vec<crate::SyscallLogEntry>,
    pub upcall_queue: crate::upcall::UpcallQueue,
    pub memory_break: *const u8,
}

// KERNEL_DATA is set to Some in `fake::Kernel::new` and set to None when the
// `fake::Kernel` is dropped.
thread_local!(pub(crate) static KERNEL_DATA: RefCell<Option<KernelData>> = const { RefCell::new(None) });

// Convenience function to get mutable access to KERNEL_DATA.
pub(crate) fn with_kernel_data<F: FnOnce(Option<&mut KernelData>) -> R, R>(f: F) -> R {
    KERNEL_DATA.with(|refcell| f(refcell.borrow_mut().as_mut()))
}

// Per-driver data stored in KernelData.
pub struct DriverData {
    pub driver: std::rc::Rc<dyn crate::fake::SyscallDriver>,
    pub num_upcalls: u32,

    // Currently-valid upcalls passed to Subscribe. The key is the subscribe
    // number.
    pub upcalls: std::collections::HashMap<u32, crate::upcall::Upcall>,
}
