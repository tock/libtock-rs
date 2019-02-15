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
    bytes: [u8; 39],
    occupied: usize,
}

impl BlePayload {
    pub fn add(&mut self, kind: u8, content: &[u8]) -> Result<(), ()> {
        self.check_can_write_num_bytes(content.len() + 2)?;

        self.bytes[self.occupied] = (content.len() + 1) as u8;
        self.bytes[self.occupied + 1] = kind;
        let write = &mut self.bytes[self.occupied + 2..(self.occupied + content.len() + 2)];
        write.clone_from_slice(content);
        self.occupied += 2 + content.len();
        Ok(())
    }

    pub fn add_flag(&mut self, flag: u8) -> Result<(), ()> {
        self.check_can_write_num_bytes(3)?;

        self.bytes[self.occupied] = 2;
        self.bytes[self.occupied + 1] = gap_types::FLAGS;
        self.bytes[self.occupied + 2] = flag;
        self.occupied += 3;
        Ok(())
    }

    pub fn new() -> Self {
        BlePayload {
            bytes: [0; 39],
            occupied: 0,
        }
    }

    pub fn add_service_payload(&mut self, uuid: [u8; 2], content: &[u8]) -> Result<(), ()> {
        self.check_can_write_num_bytes(4 + content.len())?;
        self.bytes[self.occupied] = (content.len() + 3) as u8;
        self.bytes[self.occupied + 1] = gap_types::SERVICE_DATA;
        self.bytes[self.occupied + 2] = uuid[0];
        self.bytes[self.occupied + 3] = uuid[1];

        let write = &mut self.bytes[self.occupied + 4..(self.occupied + content.len() + 4)];
        write.clone_from_slice(content);
        self.occupied += 4 + content.len();
        Ok(())
    }

    fn check_can_write_num_bytes(&self, number: usize) -> Result<(), ()> {
        if self.occupied + number <= self.bytes.len() {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl AsRef<[u8]> for BlePayload {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[0..self.occupied]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let mut pld = BlePayload::new();
        pld.add(1, &[2]);
        assert_eq!(pld.as_ref().len(), 3);
        assert_eq!(pld.as_ref(), &mut [2, 1, 2])
    }

    #[test]
    fn test_add_service_payload() {
        let mut pld = BlePayload::new();
        pld.add_service_payload([1, 2], &[2]);
        assert_eq!(pld.as_ref(), &[4, 0x16, 1, 2, 2])
    }

    #[test]
    fn test_add_service_payload_two_times() {
        let mut pld = BlePayload::new();
        pld.add_service_payload([1, 2], &[2]);
        pld.add_service_payload([1, 2], &[2, 3]);

        assert_eq!(pld.as_ref(), &[4, 0x16, 1, 2, 2, 5, 0x16, 1, 2, 2, 3])
    }

    #[test]
    fn big_data_causes_error() {
        let mut pld = BlePayload::new();
        assert!(pld.add_service_payload([1, 2], &[0; 36]).is_err());
    }

    #[test]
    fn initial_size_is_zero() {
        let pld = BlePayload::new();
        assert_eq!(pld.as_ref().len(), 0);
    }
}
