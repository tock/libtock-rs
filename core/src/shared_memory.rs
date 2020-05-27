use crate::syscalls;
use core::cell::UnsafeCell;
use core::ptr;

#[must_use = "Shared memory risks being dropped too early. Drop it manually."]
pub struct SharedMemory<'a> {
    driver_number: usize,
    allow_number: usize,
    buffer_to_share: UnsafeCell<&'a mut [u8]>,
}

impl<'a> SharedMemory<'a> {
    pub fn new(
        driver_number: usize,
        allow_number: usize,
        buffer_to_share: &'a mut [u8],
    ) -> SharedMemory<'a> {
        SharedMemory {
            driver_number,
            allow_number,
            buffer_to_share: UnsafeCell::new(buffer_to_share),
        }
    }

    pub fn read_bytes<D: AsMut<[u8]>>(&self, mut destination: D) {
        self.operate_on_mut(|buf| safe_copy(buf, destination.as_mut()));
    }

    pub fn write_bytes<S: AsRef<[u8]>>(&mut self, source: S) {
        self.operate_on_mut(|buf| safe_copy(source.as_ref(), buf));
    }

    pub(crate) fn operate_on_mut<R, F: FnOnce(&mut [u8]) -> R>(&self, func: F) -> R {
        unsafe { func(&mut *self.buffer_to_share.get()) }
    }
}

impl Drop for SharedMemory<'_> {
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
