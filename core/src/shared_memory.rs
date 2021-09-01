use libtock_platform::RawSyscalls;
use libtock_runtime::TockSyscalls;

#[must_use = "Shared memory risks being dropped too early. Drop it manually."]
pub struct SharedMemory<'a> {
    driver_number: usize,
    allow_number: usize,
    buffer_to_share: &'a mut [u8],
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
            buffer_to_share,
        }
    }

    pub fn read_bytes<T: AsMut<[u8]>>(&self, mut destination: T) {
        safe_copy(self.buffer_to_share, destination.as_mut());
    }

    pub fn write_bytes<T: AsRef<[u8]>>(&mut self, source: T) {
        safe_copy(source.as_ref(), self.buffer_to_share);
    }
}

impl<'a> Drop for SharedMemory<'a> {
    fn drop(&mut self) {
        unsafe {
            TockSyscalls::syscall4::<3>([
                self.driver_number.into(),
                self.allow_number.into(),
                //(ptr::null_mut() as u32).into(), // rust gets mad
                (0 as u32).into(),
                (0 as u32).into(),
            ]);
        }
    }
}

fn safe_copy(origin: &[u8], destination: &mut [u8]) {
    let amount = origin.len().min(destination.len());
    let origin = &origin[0..amount];
    let destination = &mut destination[0..amount];
    destination.copy_from_slice(origin);
}
