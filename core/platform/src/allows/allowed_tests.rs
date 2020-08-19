use crate::Allowed;
use core::marker::PhantomData;
use core::ptr::NonNull;

// How do we simulate accesses to the shared buffer by the kernel?
//
// Well, a naive way would be to mutate the `buffer` variable directly. Because
// Allowed accesses the memory through a *mut pointer, such a test would compile
// and run fine with the current Rust compiler. As far as I can tell, it would
// not hit any behavior documented as undefined at
// https://doc.rust-lang.org/stable/reference/behavior-considered-undefined.html,
// nor would it cause rustc to generate LLVM bitcode that encounters undefined
// behavior.
//
// However, the naive approach will throw an "undefined behavior" error when run
// under Miri (e.g. with `cargo miri test`), which uses the stacked borrows
// model [1]. In particular, accessing the `buffer` variable directly pops the
// SharedRW off buffer's borrow stack, which prevents Allowed from using its
// *mut pointer to access `buffer` afterwards. It is likely that Rust will adopt
// the stacked borrows model as its formal model for borrow validity, and in
// that case accessing `buffer` in that manner will become undefined behavior.
// In addition, running these tests under Miri is highly valuable, as this is
// tricky code to get correct and an unsound API may be hard to fix.
//
// Instead, we explicitly refer to buffer through use of a KernelPtr that
// simulates the pointer that `allow()` would hand to the Tock kernel. As far as
// the stacked borrows model is concerned, accesses through the KernelPtr
// variable behave identically to mutations performed by the kernel. This
// pattern allows us to use `cargo miri test` to not only execute the unit
// tests, but to test whether Allowed would encounter undefined behavior when
// interacting with a real Tock kernel.
//
// [1] https://plv.mpi-sws.org/rustbelt/stacked-borrows/paper.pdf
struct KernelPtr<'b, T: Copy + 'b> {
    ptr: NonNull<T>,

    // We need to consume the 'b lifetime. This is very similar to Allowed's
    // implementation.
    _phantom: PhantomData<&'b mut T>,
}

impl<'b, T: Copy + 'b> KernelPtr<'b, T> {
    // The constructor for KernelPtr; simulates allow(). Returns both the
    // Allowed instance the Platform would return and a KernelPtr the test can
    // use to simulate a kernel.
    pub fn allow(buffer: &'b mut T) -> (Allowed<'b, T>, KernelPtr<'b, T>) {
        let ptr = NonNull::new(buffer).unwrap();
        // Discard buffer *without* creating a reference to it, as would be done
        // if we called drop().
        let _ = buffer;
        // All 3 preconditions of Allowed::new are satisfied by the fact that
        // `buffer` is directly derived from a &'b mut T.
        let allowed = unsafe { Allowed::new(ptr) };
        let kernel_ptr = KernelPtr {
            ptr,
            _phantom: PhantomData,
        };
        (allowed, kernel_ptr)
    }

    // Replaces the value in the buffer with a new one.
    pub fn set(&self, value: T) {
        unsafe {
            core::ptr::write(self.ptr.as_ptr(), value);
        }
    }

    // Copies the contained value out of the buffer.
    pub fn get(&self) -> T {
        unsafe { core::ptr::read(self.ptr.as_ptr()) }
    }
}

#[test]
fn set() {
    let mut buffer = 1;
    let (allowed, kernel_ptr) = KernelPtr::allow(&mut buffer);
    assert_eq!(kernel_ptr.get(), 1);

    // Simulate the kernel replacing the value in buffer.
    kernel_ptr.set(2);
    allowed.set(3);
    assert_eq!(kernel_ptr.get(), 3);
}

#[test]
fn get() {
    let mut buffer = 1;
    let (allowed, kernel_ptr) = KernelPtr::allow(&mut buffer);
    assert_eq!(kernel_ptr.get(), 1);

    assert_eq!(allowed.get(), 1);
    assert_eq!(kernel_ptr.get(), 1);

    kernel_ptr.set(2);
    assert_eq!(allowed.get(), 2);
    assert_eq!(kernel_ptr.get(), 2);
}
