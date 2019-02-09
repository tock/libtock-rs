use alloc::vec::Vec;

pub mod gap_types {
    pub static COMPLETE_LOCAL_NAME: u8 = 0x09;
    pub static SERVICE_DATA: u8 = 0x16;
    pub static UUID: u8 = 0x02;
    pub static FLAGS: u8 = 0x01;
}

pub mod flags {
    pub static LE_GENERAL_DISCOVERABLE: u8 = 0x01;
}

pub struct BlePayload {
    pub bytes: Vec<u8>,
}

impl BlePayload {
    pub fn add(&mut self, kind: u8, content: &[u8]) {
        self.bytes.push((content.len() + 1) as u8);
        self.bytes.push(kind);
        for e in content {
            self.bytes.push(*e);
        }
    }

    pub fn add_flag(&mut self, flag: u8) {
        self.bytes.push(2);
        self.bytes.push(gap_types::FLAGS);
        self.bytes.push(flag);
    }

    pub fn new() -> Self {
        BlePayload { bytes: Vec::new() }
    }

    pub fn add_service_payload(&mut self, uuid: [u8; 2], content: &[u8]) {
        self.bytes.push((content.len() + 3) as u8);
        self.bytes.push(gap_types::SERVICE_DATA);
        self.bytes.push(uuid[0]);
        self.bytes.push(uuid[1]);
        for e in content {
            self.bytes.push(*e);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use alloc::vec;

    #[test]
    pub fn test_add() {
        let mut pld = BlePayload::new();
        pld.add(1, &[2]);
        assert_eq!(pld.bytes, vec![2, 1, 2])
    }

    #[test]
    pub fn test_add_service_payload() {
        let mut pld = BlePayload::new();
        pld.add_service_payload([1, 2], &[2]);
        assert_eq!(pld.bytes, &[4, 0x16, 1, 2, 2])
    }
}
