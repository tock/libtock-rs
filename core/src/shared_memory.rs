use crate::syscalls;
use core::cell::UnsafeCell;
use core::ptr;

#[must_use = "Shared memory risks being dropped too early. Drop it manually."]
pub struct SharedMemory<T> {
    driver_number: usize,
    allow_number: usize,
    buffer_to_share: UnsafeCell<T>,
}

impl<T> SharedMemory<T>
where
    T: AsMut<[u8]>,
{
    pub fn new(driver_number: usize, allow_number: usize, buffer_to_share: T) -> SharedMemory<T> {
        SharedMemory {
            driver_number,
            allow_number,
            buffer_to_share: UnsafeCell::new(buffer_to_share),
        }
    }

    pub fn read_bytes<D: AsMut<[u8]>>(&self, mut destination: D) {
        let buf = unsafe { (*self.buffer_to_share.get()).as_mut() };
        safe_copy(buf, destination.as_mut());
    }

    pub fn write_bytes<S: AsRef<[u8]>>(&mut self, source: S) {
        let buf = unsafe { (*self.buffer_to_share.get()).as_mut() };
        safe_copy(source.as_ref(), buf);
    }

    pub(crate) unsafe fn operate_on_mut_ptr<R: Sized, F: FnOnce(*mut u8) -> R>(
        &self,
        func: F,
    ) -> R {
        func((*self.buffer_to_share.get()).as_mut().as_mut_ptr())
    }
}

impl<T> Drop for SharedMemory<T> {
    fn drop(&mut self) {
        unsafe {
            syscalls::raw::allow(self.driver_number, self.allow_number, ptr::null_mut(), 0);
        }
    }
}

fn safe_copy(origin: &[u8], destination: &mut [u8]) {
    let amount = origin.len().min(destination.len());
    let origin = &origin[0..amount];
    let destination = &mut destination[0..amount];
    destination.clone_from_slice(origin);
}
