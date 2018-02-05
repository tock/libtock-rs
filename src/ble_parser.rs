use alloc::*;
use simple_ble::BUFFER_SIZE_SCAN;

pub fn find(buffer: &[u8; BUFFER_SIZE_SCAN], kind: u8) -> Option<Vec<&u8>> {
    let mut iter = buffer[8..BUFFER_SIZE_SCAN].iter();

    loop {
        match iter.next() {
            Some(&len) => {
                let data_type = iter.next();
                if data_type == Some(&kind) {
                    return Some(iter.take(len as usize - 1).collect::<Vec<&u8>>());
                } else {
                    if len > 0 {
                        for i in 0..len - 1 {
                            iter.next();
                        }
                    } else {
                        return None;
                    }
                }
            }
            None => return None,
        }
    }
}
#[cfg(test)]
mod test {
    use ble_parser::*;

    #[test]
    pub fn extracts_data_for_ids_correctly() {
        let mut buf = [0; BUFFER_SIZE_SCAN];
        {
            let mut slice = &mut buf[8..23];
            let data = &[
                0x02, 0x02, 0x01, 0x02, 0x01, 0x03, 0x03, 0x16, 0x01, 0x02, 0x04, 0xFF, 0x01, 0x02,
                0x03,
            ];
            slice.clone_from_slice(data);
        }
        assert_eq!(find(&buf, 0x02), Some(vec![&0x01]));
        assert_eq!(find(&buf, 0x01), Some(vec![&0x03]));
        assert_eq!(find(&buf, 0x16), Some(vec![&0x01, &0x02]));
        assert_eq!(find(&buf, 0xFF), Some(vec![&0x01, &0x02, &0x03]));
    }

    #[test]
    pub fn doesnt_panic_for_defect_packets() {
        let mut buf = [0; BUFFER_SIZE_SCAN];
        {
            let mut slice = &mut buf[8..18];
            let data = &[0x02, 0x02, 0x00, 0x00, 0x00, 0x00, 0x0, 0x16, 0x01, 0x02];
            slice.clone_from_slice(data);
        }
    }
}
