use alloc::boxed::Box;
use alloc::String;
use ipc::Client;

#[repr(u32)]
pub enum ReadingType {
    Temperature = 0,
    Humidity = 1,
    Light = 2
}

pub struct BleEss {
    client: Client,
    buffer: Box<[u8]>
}

pub fn connect() -> Result<BleEss, ()> {
    let mut client = Client::new(
                String::from("org.tockos.services.ble-ess"))?;
    let buffer = client.share(5)?;
    Ok(BleEss {
        client: client,
        buffer: buffer
    })
}

impl BleEss {
    pub fn set_reading<I>(&mut self, sensor: ReadingType, data: I) -> Result<(), ()>
            where I: Into<i32> {
        let sensor_type = sensor as u32;
        let data = Into::<i32>::into(data) as u32;
        self.buffer[0..4].copy_from_slice(&[(sensor_type & 0xff) as u8,
                                        ((sensor_type >> 8) & 0xff) as u8,
                                        ((sensor_type >> 16) & 0xff) as u8,
                                        ((sensor_type >> 24) & 0xff) as u8]);
        self.buffer[4..8].copy_from_slice(&[(data & 0xff) as u8,
                                    ((data >> 8) & 0xff) as u8,
                                    ((data >> 16) & 0xff) as u8,
                                    ((data >> 24) & 0xff) as u8]);
        self.client.notify()
    }
}
