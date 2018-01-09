extern crate alloc;
use alloc::{String, Vec};
use syscalls::{allow, command, allow16};

const DRIVER_NUMBER: u32 = 0x30000;

mod ble_commands {
    pub const START_ADVERTISING: u32 = 0;
    pub const SET_ADVERTISING_INTERVAL: u16 = 3;
    pub const ALLOW_ADVERTISMENT_BUFFER: u32 = 0x32;
    pub const REQ_ADV_ADDRESS: u32 = 6;
}

mod gap_flags {
    pub const BLE_DISCOVERABLE: u8 = 0x02;
    pub const BLE_NOT_DISCOVERABLE: u8 = 0x01;
    pub const ONLY_LE: u8 = 0x04;
}

mod gap_data {
    pub const COMPLETE_LIST_16BIT_SERVICE_IDS: u32 = 0x03;
    pub const COMPLETE_LOCAL_NAME: u32 = 0x09;
    pub const SET_FLAGS: u32 = 1;
}

pub struct BleDeviceUninitialized {
    interval: u16,
    name: String,
    uuid: Vec<u16>,
    flags: Vec<u8>,
    buffer: [u8; 39],
}

pub struct BleDeviceAdvertising {
    interval: u16,
    name: String,
    uuid: Vec<u16>,
    flags: Vec<u8>,
    buffer: [u8; 39],
}

pub struct BleDeviceInitialized {
    interval: u16,
    name: String,
    uuid: Vec<u16>,
    flags: Vec<u8>,
    buffer: [u8; 39],
}

impl BleDeviceUninitialized {
    pub fn new(
        interval: u16,
        name: String,
        uuid: Vec<u16>,
        stay_visible: bool,
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
        }
    }
    pub fn initialize(self) -> Result<BleDeviceInitialized, &'static str> {
        Ok(self)
            .and_then(|ble| ble.set_advertising_buffer())
            .and_then(|ble| ble.set_advertising_interval())
            .and_then(|ble| ble.set_advertising_flags())
            .and_then(|ble| ble.request_address())
            .and_then(|ble| ble.set_advertsing_uuid())
            .and_then(|ble| ble.set_advertising_name())
            .and_then(|ble| {
                Ok(BleDeviceInitialized {
                    interval: ble.interval,
                    name: ble.name,
                    uuid: ble.uuid,
                    flags: ble.flags,
                    buffer: ble.buffer,
                })
            })
    }

    fn set_advertising_interval(self) -> Result<Self, &'static str> {
        match unsafe {
            command(
                DRIVER_NUMBER,
                ble_commands::SET_ADVERTISING_INTERVAL as u32,
                self.interval as isize,
                0,
            )
        } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn request_address(self) -> Result<Self, &'static str> {
        match unsafe { command(DRIVER_NUMBER, ble_commands::REQ_ADV_ADDRESS, 0, 0) } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertising_name(self) -> Result<Self, &'static str> {
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

    fn set_advertsing_uuid(self) -> Result<Self, &'static str> {
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

    fn set_advertising_flags(self) -> Result<Self, &'static str> {
        match unsafe { allow(DRIVER_NUMBER, gap_data::SET_FLAGS, &self.flags) } {
            0 => Ok(self),
            _ => Err(""),
        }
    }

    fn set_advertising_buffer(self) -> Result<Self, &'static str> {
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
}

impl BleDeviceInitialized {
    pub fn start_advertising(self) -> Result<BleDeviceAdvertising, &'static str> {
        match unsafe { command(DRIVER_NUMBER, ble_commands::START_ADVERTISING, 0, 0) } {
            0 => Ok(BleDeviceAdvertising {
                interval: self.interval,
                name: self.name,
                uuid: self.uuid,
                flags: self.flags,
                buffer: self.buffer,
            }),
            _ => Err(""),
        }
    }
}
