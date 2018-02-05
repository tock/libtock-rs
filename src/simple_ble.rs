#[allow(unused_extern_crates)]
extern crate alloc;
use alloc::{String, Vec};
use fmt;
use syscalls::{allow, command, subscribe, allow16};

const DRIVER_NUMBER: usize = 0x30000;
const MAX_PAYLOAD_SIZE: usize = 2;
pub const BUFFER_SIZE: usize = 39;
pub const BUFFER_SIZE_SCAN: usize = 39;

mod ble_commands {
    pub const START_ADVERTISING: usize = 0;
    pub const SET_ADVERTISING_INTERVAL: usize = 3;
    pub const ALLOW_ADVERTISMENT_BUFFER: usize = 0x32;
    pub const REQ_ADV_ADDRESS: usize = 6;
    pub const BLE_PASSIVE_SCAN_SUB: usize = 0;
    pub const ALLOW_SCAN_BUFFER: usize = 0x31;
    pub const PASSIVE_SCAN: usize = 5;
}

mod gap_flags {
    pub const BLE_DISCOVERABLE: u8 = 0x02;
    pub const BLE_NOT_DISCOVERABLE: u8 = 0x01;
    pub const ONLY_LE: u8 = 0x04;
}

mod gap_data {
    pub const COMPLETE_LIST_16BIT_SERVICE_IDS: usize = 0x03;
    pub const COMPLETE_LOCAL_NAME: usize = 0x09;
    pub const SET_FLAGS: usize = 1;
    pub const SERVICE_DATA: usize = 0x16;
}

#[allow(dead_code)]
pub struct BleDeviceUninitialized {
    interval: u16,
    name: String,
    uuid: Vec<u16>,
    flags: Vec<u8>,
    buffer: [u8; BUFFER_SIZE],
    service_payload: [u8; MAX_PAYLOAD_SIZE + 2],
    temperature: u16,
}

#[allow(dead_code)]
pub struct BleDeviceInitialized<'a> {
    interval: &'a mut u16,
    name: &'a mut String,
    uuid: &'a mut Vec<u16>,
    flags: &'a mut Vec<u8>,
    buffer: &'a mut [u8; 39],
    service_payload: &'a mut [u8; MAX_PAYLOAD_SIZE + 2],
    temperature: u16,
}

#[allow(dead_code)]
pub struct BleDeviceAdvertising<'a> {
    interval: &'a mut u16,
    name: &'a mut String,
    uuid: &'a mut Vec<u16>,
    flags: &'a mut Vec<u8>,
    buffer: &'a mut [u8; 39],
    service_payload: &'a mut [u8; MAX_PAYLOAD_SIZE + 2],
    temperature: u16,
}

#[allow(dead_code)]
impl<'a> BleDeviceUninitialized {
    pub fn new(
        interval: u16,
        name: String,
        uuid: Vec<u16>,
        stay_visible: bool,
        temperature: u16,
    ) -> BleDeviceUninitialized {
        let flags: [u8; 1] = [
            gap_flags::ONLY_LE | (if stay_visible {
                gap_flags::BLE_DISCOVERABLE
            } else {
                gap_flags::BLE_NOT_DISCOVERABLE
            }),
        ];

        BleDeviceUninitialized {
            interval: interval,
            name: name,
            uuid: uuid,
            flags: flags.to_vec(),
            buffer: [0; 39],
            service_payload: [0; MAX_PAYLOAD_SIZE + 2],
            temperature: temperature,
        }
    }
    pub fn initialize(&'a mut self) -> Result<BleDeviceInitialized<'a>, &'static str> {
        Ok(self)
            .and_then(|ble| ble.set_advertising_buffer())
            .and_then(|ble| ble.request_address())
            .and_then(|ble| ble.set_advertising_interval())
            .and_then(|ble| ble.set_advertising_flags())
            .and_then(|ble| ble.set_advertising_name())
            .and_then(|ble| ble.set_advertsing_uuid())
            .and_then(|ble| ble.set_service_payload())
            .and_then(|ble| {
                Ok(BleDeviceInitialized {
                    interval: &mut ble.interval,
                    name: &mut ble.name,
                    uuid: &mut ble.uuid,
                    flags: &mut ble.flags,
                    buffer: &mut ble.buffer,
                    service_payload: &mut ble.service_payload,
                    temperature: ble.temperature,
                })
            })
    }

    fn set_advertising_interval(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe {
            command(
                DRIVER_NUMBER,
                ble_commands::SET_ADVERTISING_INTERVAL,
                self.interval as usize,
                0,
            )
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn request_address(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe { command(DRIVER_NUMBER, ble_commands::REQ_ADV_ADDRESS, 0, 0) } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertising_name(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe {
            allow(
                DRIVER_NUMBER,
                gap_data::COMPLETE_LOCAL_NAME,
                self.name.as_bytes(),
            )
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertsing_uuid(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe {
            allow16(
                DRIVER_NUMBER,
                gap_data::COMPLETE_LIST_16BIT_SERVICE_IDS,
                &self.uuid,
            )
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertising_flags(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe { allow(DRIVER_NUMBER, gap_data::SET_FLAGS, &self.flags) } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertising_buffer(&mut self) -> Result<&mut Self, &'static str> {
        match unsafe {
            allow(
                DRIVER_NUMBER,
                ble_commands::ALLOW_ADVERTISMENT_BUFFER,
                &self.buffer,
            )
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_service_payload(&mut self) -> Result<&mut Self, &'static str> {
        {
            let payload = &mut self.service_payload[2..MAX_PAYLOAD_SIZE + 2];
            payload.clone_from_slice(&[0; MAX_PAYLOAD_SIZE]);
        }
        self.service_payload[1] = 0x18;
        self.service_payload[0] = 0x09;
        {
            let payload = &mut self.service_payload[2..4];
            payload.clone_from_slice(&fmt::convert_le(self.temperature));
        }
        match unsafe { allow(DRIVER_NUMBER, gap_data::SERVICE_DATA, &self.service_payload) } {
            0 => Ok(self),
            _ => Err(""),
        }
    }
}

impl<'a> BleDeviceInitialized<'a> {
    pub fn start_advertising(&mut self) -> Result<BleDeviceAdvertising, &'static str> {
        match unsafe { command(DRIVER_NUMBER, ble_commands::START_ADVERTISING, 0, 0) } {
            0 => Ok(BleDeviceAdvertising {
                interval: &mut self.interval,
                name: &mut self.name,
                uuid: &mut self.uuid,
                flags: &mut self.flags,
                buffer: &mut self.buffer,
                service_payload: &mut self.service_payload,
                temperature: self.temperature,
            }),
            _ => Err(""),
        }
    }
}

#[allow(dead_code)]
pub struct BleScanning<'a, CB: Callback> {
    buffer: &'a [u8; BUFFER_SIZE_SCAN],
    callback: CB,
}

pub trait Callback {
    fn callback(&mut self, usize, usize);
}
impl<F: FnMut(usize, usize)> Callback for F {
    fn callback(&mut self, result: usize, len: usize) {
        self(result, len);
    }
}

impl<'a, CB: Callback> BleScanning<'a, CB> {
    pub fn start(
        buffer: &[u8; BUFFER_SIZE_SCAN],
        callback: CB,
    ) -> Result<BleScanning<CB>, &'static str> {
        extern "C" fn cb<CB: Callback>(result: usize, len: usize, _: usize, ud: usize) {
            let mut callback = unsafe { &mut *(ud as *mut CB) };
            callback.callback(result, len);
        }
        let mut ble = BleScanning {
            buffer: buffer,
            callback: callback,
        };

        unsafe {
            subscribe(
                DRIVER_NUMBER,
                ble_commands::BLE_PASSIVE_SCAN_SUB,
                cb::<CB>,
                &mut ble.callback as *mut _ as usize,
            );
            allow(DRIVER_NUMBER, ble_commands::ALLOW_SCAN_BUFFER, buffer);
            command(DRIVER_NUMBER, ble_commands::PASSIVE_SCAN, 1, 0);
        }
        Ok(ble)
    }
}
