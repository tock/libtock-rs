// The thread_local module contains a reference to this thread's Kernel instance
// (if one exists). It provides functionality to create and manage this
// instance, as well as some instrumentation to help identify when tests leak a
// Rc<Kernel>.

use crate::fake::Kernel;
use std::rc::{Rc, Weak};

// Clears this thread's Kernel instance. This allows the Kernel to be
// deallocated, and should be called by Kernel's Drop implementation.
pub fn clear_kernel() {
    THREAD_KERNEL.with(|thread_kernel| thread_kernel.kernel.replace(Weak::new()));
}

// Retrieves this thread's Kernel instance, if one is available.
pub fn get_kernel() -> Option<Rc<Kernel>> {
    let clone = THREAD_KERNEL.with(|thread_kernel| {
        let weak = thread_kernel.kernel.replace(Weak::new());
        let clone = weak.clone();
        thread_kernel.kernel.replace(weak);
        clone
    });
    clone.upgrade()
}

// Sets this thread's Kernel instance. If this thread already has a Kernel
// instance, this panics with a message indicating the name of the existing
// Kernel instance, as it was presumably leaked.
pub fn set_kernel(kernel: &Rc<Kernel>) {
    THREAD_KERNEL.with(|thread_kernel| {
        let existing_weak = thread_kernel.kernel.replace(Rc::downgrade(kernel));
        if let Some(existing_kernel) = existing_weak.upgrade() {
            existing_kernel.report_leaked();
        }
    });
}

// Reference to this thread's Kernel instance, if one has been initialized. This
// is a weak reference so that when a unit test is done with its Kernel, the
// following cleanup can happen:
//   1. The test drops its Rc<Kernel> instances.
//   2. The strong count drops to 0 so the Kernel is dropped.
//   3. Kernel's Drop implementation clears out THREAD_KERNEL, removing the weak
//      reference.
//   4. The backing storage holding the Kernel is deallocated.
thread_local!(static THREAD_KERNEL: ThreadKernelRef = ThreadKernelRef::new());

// Type that wraps a Weak<Kernel> pointing to this thread's Kernel. This
// wrapper's Drop implementation verifies the Kernel has been cleaned up, in
// order to help identify leaked Kernels. Note that ThreadKernelRef's Drop is
// not *guaranteed* to run; see std::thread::LocalKey's documentation for more
// details.
struct ThreadKernelRef {
    kernel: std::cell::Cell<Weak<Kernel>>,
}

impl ThreadKernelRef {
    pub fn new() -> ThreadKernelRef {
        // Note that Weak::new() does not allocate, while Default::default()
        // does.
        ThreadKernelRef {
            kernel: std::cell::Cell::new(Weak::new()),
        }
    }
}

impl Drop for ThreadKernelRef {
    fn drop(&mut self) {
        if let Some(leaked_kernel) = self.kernel.replace(Weak::new()).upgrade() {
            leaked_kernel.report_leaked();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests a sequence of clear_kernel, get_kernel, and set_kernel calls
    // representing a fairly typical unit test (with a few extra get_kernel()
    // invocations for extra verification).
    #[test]
    fn normal_test() {
        assert!(get_kernel().is_none());
        let rc_kernel = Kernel::new();
        assert!(Rc::ptr_eq(
            &get_kernel().expect("get_kernel returned None"),
            &rc_kernel
        ));
        clear_kernel();
        assert!(get_kernel().is_none());
    }

    // Tests a sequence of calls that looks like a leak; specifically, create a
    // second kernel before the first kernel is cleared.
    #[test]
    fn duplicate_kernel() {
        assert!(get_kernel().is_none());
        #[rustfmt::skip]
        let (rc_kernel_1, new_location) = (Kernel::new(), std::panic::Location::caller());
        assert!(Rc::ptr_eq(
            &get_kernel().expect("get_kernel returned None"),
            &rc_kernel_1
        ));
        let result = std::panic::catch_unwind(|| {
            Kernel::new();
        });
        let panic_arg = result.expect_err("Setting a duplicate kernel did not panic");
        let message = panic_arg
            .downcast_ref::<String>()
            .expect("Wrong panic payload type");
        // Verify the panic message mentions the correct kernel.
        assert!(message.contains(&format!("{}:{}", new_location.file(), new_location.line())));
    }

    // Verifies that ThreadKernelRef's Drop implementations detects a leaked
    // Kernel at thread exit. We unfortunately cannot test this by spawning a
    // thread and leaking a Kernel, because panicing from TLS destructors fails:
    // https://github.com/rust-lang/rust/issues/24479
    #[test]
    fn thread_drop_leak() {
        let result = std::panic::catch_unwind(|| {
            let thread_ref = ThreadKernelRef::new();
            // Create a kernel, loading it into THREAD_KERNEL, then use
            // Cell::replace() to move the kernel reference into thread_ref.
            let _rc_kernel = Kernel::new();
            thread_ref
                .kernel
                .replace(THREAD_KERNEL.with(|tk| tk.kernel.replace(Weak::new())));
            // Drop the ThreadKernelRef, simulating a thread exit.
            drop(thread_ref);
        });
        let panic_arg = result.expect_err("Leaking a thread's kernel did not panic");
        let message = panic_arg
            .downcast_ref::<String>()
            .expect("Wrong panic payload type");
        // Verify the panic message is from Kernel::report_leaked.
        assert!(message.contains("perhaps a Rc<Kernel> was leaked"));
    }
}
