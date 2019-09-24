pub fn find(buffer: &[u8], kind: u8) -> Option<&[u8]> {
    let mut iter = buffer[8..].iter().enumerate();
    let buffer_len = buffer.len();

    loop {
        match iter.next() {
            Some((_, &len)) => match iter.next() {
                Some((i, potentialkind)) => {
                    if potentialkind == &kind {
                        if (8 + i) + len as usize > buffer_len {
                            return None;
                        } else {
                            return Some(&buffer[9 + i..8 + i + len as usize]);
                        }
                    } else if len > 0 {
                        for _ in 0..len - 1 {
                            iter.next();
                        }
                    } else {
                        return None;
                    }
                }
                _ => return None,
            },
            None => return None,
        }
    }
}

pub fn extract_for_service(service: [u8; 2], data: &[u8]) -> Option<&[u8]> {
    if data.len() > 1 {
        if service[0] == data[0] && service[1] == data[1] {
            Some(&data[2..])
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::ble_parser::*;
    use crate::simple_ble::BUFFER_SIZE_SCAN;

    #[test]
    fn extracts_data_for_ids_correctly() {
        let mut buf = [0; BUFFER_SIZE_SCAN];
        {
            let slice = &mut buf[8..23];
            let data = &[
                0x02, 0x02, 0x01, 0x02, 0x01, 0x03, 0x03, 0x16, 0x01, 0x02, 0x04, 0xFF, 0x01, 0x02,
                0x03,
            ];
            slice.clone_from_slice(data);
        }
        assert_eq!(find(&buf, 0x02), Some(&[0x01][0..1]));
        assert_eq!(find(&buf, 0x01), Some(&[0x03][0..1]));
        assert_eq!(find(&buf, 0x16), Some(&[0x01, 0x02][0..2]));
        assert_eq!(find(&buf, 0xFF), Some(&[0x01, 0x02, 0x03][0..3]));
    }

    #[test]
    fn doesnt_panic_for_defect_packets() {
        let mut buf = [0; BUFFER_SIZE_SCAN];
        {
            let slice = &mut buf[8..18];
            let data = &[0x02, 0x02, 0x00, 0x00, 0x00, 0x00, 0x0, 0x16, 0x01, 0x02];
            slice.clone_from_slice(data);
        }
    }

    #[test]
    fn ignores_illegal_lengths_in_packets() {
        let mut buf = [0; 11];
        {
            let slice = &mut buf[8..10];
            let data = &[0x04, 0x02];
            slice.clone_from_slice(data);
        }
        assert_eq!(find(&buf, 0xF2), None);
    }
}
