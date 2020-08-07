use crate::{AllowedSlice, OutOfBounds};
use core::marker::PhantomData;
use core::ptr::NonNull;

// KernelSlice simulates a kernel's access to a slice that has been shared with
// the kernel. It is the slice-based equivalent to KernelPtr, defined in
// allowed_tests.rs. See KernelPtr's documentation for a description of
// KernelSlice's purpose.
struct KernelSlice<'b, T: Copy + 'b> {
    data: NonNull<T>,
    len: usize,

    // Consume the 'b lifetime.
    _phantom: PhantomData<&'b mut [T]>,
}

impl<'b, T: Copy + 'b> KernelSlice<'b, T> {
    pub fn allow_slice(buffer: &'b mut [T]) -> (AllowedSlice<'b, T>, KernelSlice<'b, T>) {
        let data = NonNull::new(buffer.as_mut_ptr()).unwrap();
        let len = buffer.len();
        let _ = buffer;
        let allowed_slice = unsafe { AllowedSlice::new(data, len) };
        let kernel_slice = KernelSlice {
            data,
            len,
            _phantom: PhantomData,
        };
        (allowed_slice, kernel_slice)
    }

    // Copies the value out of the given entry in the buffer and returns it.
    pub fn get(&self, index: usize) -> T {
        assert!(index < self.len);
        unsafe { core::ptr::read(self.data.as_ptr().add(index)) }
    }
}

#[test]
fn set() {
    let mut buffer = [0, 1, 2];
    let (allowed_slice, kernel_slice) = KernelSlice::allow_slice(&mut buffer);
    assert_eq!(kernel_slice.get(0), 0);
    assert_eq!(kernel_slice.get(1), 1);
    assert_eq!(kernel_slice.get(2), 2);

    assert!(allowed_slice.set(4, 4) == Err(OutOfBounds));
    assert_eq!(kernel_slice.get(0), 0);
    assert_eq!(kernel_slice.get(1), 1);
    assert_eq!(kernel_slice.get(2), 2);

    assert!(allowed_slice.set(1, 4) == Ok(()));
    assert_eq!(kernel_slice.get(0), 0);
    assert_eq!(kernel_slice.get(1), 4);
    assert_eq!(kernel_slice.get(2), 2);
}

#[test]
fn get() {
    let mut buffer = [0, 1, 2];
    let (allowed_slice, kernel_slice) = KernelSlice::allow_slice(&mut buffer);
    assert_eq!(kernel_slice.get(0), 0);
    assert_eq!(kernel_slice.get(1), 1);
    assert_eq!(kernel_slice.get(2), 2);

    assert!(allowed_slice.get(4).is_err());
    assert_eq!(kernel_slice.get(0), 0);
    assert_eq!(kernel_slice.get(1), 1);
    assert_eq!(kernel_slice.get(2), 2);

    assert!(allowed_slice.get(1) == Ok(1));
    assert_eq!(kernel_slice.get(0), 0);
    assert_eq!(kernel_slice.get(1), 1);
    assert_eq!(kernel_slice.get(2), 2);
}

#[test]
fn get_or_default() {
    let mut buffer = [0, 1, 2];
    let (allowed_slice, kernel_slice) = KernelSlice::allow_slice(&mut buffer);
    assert_eq!(kernel_slice.get(0), 0);
    assert_eq!(kernel_slice.get(1), 1);
    assert_eq!(kernel_slice.get(2), 2);

    assert_eq!(allowed_slice.get_or_default(4, 3), 3);
    assert_eq!(kernel_slice.get(0), 0);
    assert_eq!(kernel_slice.get(1), 1);
    assert_eq!(kernel_slice.get(2), 2);

    assert_eq!(allowed_slice.get_or_default(1, 4), 1);
    assert_eq!(kernel_slice.get(0), 0);
    assert_eq!(kernel_slice.get(1), 1);
    assert_eq!(kernel_slice.get(2), 2);
}
