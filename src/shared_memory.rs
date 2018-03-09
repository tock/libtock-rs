pub trait ShareableMemory {
    fn driver_number(&self) -> usize;

    fn allow_number(&self) -> usize;

    fn to_bytes(&mut self) -> &mut [u8];
}

pub struct SharedMemory<SM: ShareableMemory> {
    #[allow(dead_code)] // Used in drop
    pub(crate) shareable_memory: SM,
}

impl<SM: ShareableMemory> SharedMemory<SM> {
    pub fn to_bytes(&mut self) -> &mut [u8] {
        self.shareable_memory.to_bytes()
    }
}
