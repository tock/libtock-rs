use alloc::String;
use ipc::IPCBuffer;
use ipc::ServerHandle;
use result::TockResult;
use shared_memory::SharedMemory;

#[repr(u32)]
pub enum ReadingType {
    Temperature = 0,
    Humidity = 1,
    Light = 2,
}

pub struct BleEss<'a> {
    server: ServerHandle,
    pub memory: SharedMemory<'a>,
}

pub fn connect(buffer: &mut IPCBuffer) -> Result<BleEss, ()> {
    let server =
        ServerHandle::discover_service(String::from("org.tockos.services.ble-ess")).unwrap();
    let memory = server.share(buffer).unwrap();
    Ok(BleEss { server, memory })
}

impl<'a> BleEss<'a> {
    pub fn set_reading<I>(&mut self, sensor: ReadingType, data: I) -> TockResult<usize>
    where
        I: Into<i32>,
    {
        let sensor_type = sensor as u32;
        let data = Into::<i32>::into(data) as u32;
        let mut buf = [0; 8];

        buf[0..4].copy_from_slice(&[
            (sensor_type & 0xff) as u8,
            ((sensor_type >> 8) & 0xff) as u8,
            ((sensor_type >> 16) & 0xff) as u8,
            ((sensor_type >> 24) & 0xff) as u8,
        ]);
        buf[4..8].copy_from_slice(&[
            (data & 0xff) as u8,
            ((data >> 8) & 0xff) as u8,
            ((data >> 16) & 0xff) as u8,
            ((data >> 24) & 0xff) as u8,
        ]);
        self.memory.write_bytes(&buf);
        self.server.notify()
    }
}
