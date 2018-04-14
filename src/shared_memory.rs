use core::ptr;
use core::slice;
use syscalls;

pub struct SharedMemory<'a> {
    pub driver_number: usize,
    pub allow_number: usize,
    pub buffer_to_share: &'a mut [u8],
}

impl<'a> SharedMemory<'a> {
    pub fn read_bytes(&self, destination: &mut [u8]) {
        destination.clone_from_slice(self.buffer_to_share);
    }

    pub fn write_bytes(&mut self, source: &[u8]) {
        self.buffer_to_share.clone_from_slice(source);
    }
}

impl<'a> Drop for SharedMemory<'a> {
    fn drop(&mut self) {
        unsafe {
            syscalls::allow_ptr(
                self.driver_number,
                self.allow_number,
                slice::from_raw_parts_mut(ptr::null_mut(), 0),
            );
        }
    }
}
