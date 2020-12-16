use crate::{error_code, return_type, CommandReturn};

#[test]
fn failure() {
    let command_return = CommandReturn {
        return_type: return_type::FAILURE,
        r1: error_code::RESERVE.into(),
        r2: 1002,
        r3: 1003,
    };
    assert_eq!(command_return.is_failure(), true);
    assert_eq!(command_return.is_failure_u32(), false);
    assert_eq!(command_return.is_failure_2_u32(), false);
    assert_eq!(command_return.is_failure_u64(), false);
    assert_eq!(command_return.is_success(), false);
    assert_eq!(command_return.is_success_u32(), false);
    assert_eq!(command_return.is_success_2_u32(), false);
    assert_eq!(command_return.is_success_u64(), false);
    assert_eq!(command_return.is_success_3_u32(), false);
    assert_eq!(command_return.is_success_u32_u64(), false);
    assert_eq!(command_return.get_failure(), Some(error_code::RESERVE));
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(command_return.return_type(), return_type::FAILURE);
}

#[test]
fn failure_u32() {
    let command_return = CommandReturn {
        return_type: return_type::FAILURE_U32,
        r1: error_code::OFF.into(),
        r2: 1002,
        r3: 1003,
    };
    assert_eq!(command_return.is_failure(), false);
    assert_eq!(command_return.is_failure_u32(), true);
    assert_eq!(command_return.is_failure_2_u32(), false);
    assert_eq!(command_return.is_failure_u64(), false);
    assert_eq!(command_return.is_success(), false);
    assert_eq!(command_return.is_success_u32(), false);
    assert_eq!(command_return.is_success_2_u32(), false);
    assert_eq!(command_return.is_success_u64(), false);
    assert_eq!(command_return.is_success_3_u32(), false);
    assert_eq!(command_return.is_success_u32_u64(), false);
    assert_eq!(command_return.get_failure(), None);
    assert_eq!(
        command_return.get_failure_u32(),
        Some((error_code::OFF, 1002))
    );
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(command_return.return_type(), return_type::FAILURE_U32);
}

#[test]
fn failure_2_u32() {
    let command_return = CommandReturn {
        return_type: return_type::FAILURE_2_U32,
        r1: error_code::ALREADY.into(),
        r2: 1002,
        r3: 1003,
    };
    assert_eq!(command_return.is_failure(), false);
    assert_eq!(command_return.is_failure_u32(), false);
    assert_eq!(command_return.is_failure_2_u32(), true);
    assert_eq!(command_return.is_failure_u64(), false);
    assert_eq!(command_return.is_success(), false);
    assert_eq!(command_return.is_success_u32(), false);
    assert_eq!(command_return.is_success_2_u32(), false);
    assert_eq!(command_return.is_success_u64(), false);
    assert_eq!(command_return.is_success_3_u32(), false);
    assert_eq!(command_return.is_success_u32_u64(), false);
    assert_eq!(command_return.get_failure(), None);
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(
        command_return.get_failure_2_u32(),
        Some((error_code::ALREADY, 1002, 1003))
    );
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(command_return.return_type(), return_type::FAILURE_2_U32);
}

#[test]
fn failure_u64() {
    let command_return = CommandReturn {
        return_type: return_type::FAILURE_U64,
        r1: error_code::BUSY.into(),
        r2: 0x00001002,
        r3: 0x00001003,
    };
    assert_eq!(command_return.is_failure(), false);
    assert_eq!(command_return.is_failure_u32(), false);
    assert_eq!(command_return.is_failure_2_u32(), false);
    assert_eq!(command_return.is_failure_u64(), true);
    assert_eq!(command_return.is_success(), false);
    assert_eq!(command_return.is_success_u32(), false);
    assert_eq!(command_return.is_success_2_u32(), false);
    assert_eq!(command_return.is_success_u64(), false);
    assert_eq!(command_return.is_success_3_u32(), false);
    assert_eq!(command_return.is_success_u32_u64(), false);
    assert_eq!(command_return.get_failure(), None);
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(
        command_return.get_failure_u64(),
        Some((error_code::BUSY, 0x00001003_00001002))
    );
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(command_return.return_type(), return_type::FAILURE_U64);
}

#[test]
fn success() {
    let command_return = CommandReturn {
        return_type: return_type::SUCCESS,
        r1: 1001,
        r2: 1002,
        r3: 1003,
    };
    assert_eq!(command_return.is_failure(), false);
    assert_eq!(command_return.is_failure_u32(), false);
    assert_eq!(command_return.is_failure_2_u32(), false);
    assert_eq!(command_return.is_failure_u64(), false);
    assert_eq!(command_return.is_success(), true);
    assert_eq!(command_return.is_success_u32(), false);
    assert_eq!(command_return.is_success_2_u32(), false);
    assert_eq!(command_return.is_success_u64(), false);
    assert_eq!(command_return.is_success_3_u32(), false);
    assert_eq!(command_return.is_success_u32_u64(), false);
    assert_eq!(command_return.get_failure(), None);
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(command_return.return_type(), return_type::SUCCESS);
}

#[test]
fn success_u32() {
    let command_return = CommandReturn {
        return_type: return_type::SUCCESS_U32,
        r1: 1001,
        r2: 1002,
        r3: 1003,
    };
    assert_eq!(command_return.is_failure(), false);
    assert_eq!(command_return.is_failure_u32(), false);
    assert_eq!(command_return.is_failure_2_u32(), false);
    assert_eq!(command_return.is_failure_u64(), false);
    assert_eq!(command_return.is_success(), false);
    assert_eq!(command_return.is_success_u32(), true);
    assert_eq!(command_return.is_success_2_u32(), false);
    assert_eq!(command_return.is_success_u64(), false);
    assert_eq!(command_return.is_success_3_u32(), false);
    assert_eq!(command_return.is_success_u32_u64(), false);
    assert_eq!(command_return.get_failure(), None);
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), Some(1001));
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(command_return.return_type(), return_type::SUCCESS_U32);
}

#[test]
fn success_2_u32() {
    let command_return = CommandReturn {
        return_type: return_type::SUCCESS_2_U32,
        r1: 1001,
        r2: 1002,
        r3: 1003,
    };
    assert_eq!(command_return.is_failure(), false);
    assert_eq!(command_return.is_failure_u32(), false);
    assert_eq!(command_return.is_failure_2_u32(), false);
    assert_eq!(command_return.is_failure_u64(), false);
    assert_eq!(command_return.is_success(), false);
    assert_eq!(command_return.is_success_u32(), false);
    assert_eq!(command_return.is_success_2_u32(), true);
    assert_eq!(command_return.is_success_u64(), false);
    assert_eq!(command_return.is_success_3_u32(), false);
    assert_eq!(command_return.is_success_u32_u64(), false);
    assert_eq!(command_return.get_failure(), None);
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), Some((1001, 1002)));
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(command_return.return_type(), return_type::SUCCESS_2_U32);
}

#[test]
fn success_u64() {
    let command_return = CommandReturn {
        return_type: return_type::SUCCESS_U64,
        r1: 0x00001001,
        r2: 0x00001002,
        r3: 1003,
    };
    assert_eq!(command_return.is_failure(), false);
    assert_eq!(command_return.is_failure_u32(), false);
    assert_eq!(command_return.is_failure_2_u32(), false);
    assert_eq!(command_return.is_failure_u64(), false);
    assert_eq!(command_return.is_success(), false);
    assert_eq!(command_return.is_success_u32(), false);
    assert_eq!(command_return.is_success_2_u32(), false);
    assert_eq!(command_return.is_success_u64(), true);
    assert_eq!(command_return.is_success_3_u32(), false);
    assert_eq!(command_return.is_success_u32_u64(), false);
    assert_eq!(command_return.get_failure(), None);
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), Some(0x00001002_00001001));
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(command_return.return_type(), return_type::SUCCESS_U64);
}

#[test]
fn success_3_u32() {
    let command_return = CommandReturn {
        return_type: return_type::SUCCESS_3_U32,
        r1: 1001,
        r2: 1002,
        r3: 1003,
    };
    assert_eq!(command_return.is_failure(), false);
    assert_eq!(command_return.is_failure_u32(), false);
    assert_eq!(command_return.is_failure_2_u32(), false);
    assert_eq!(command_return.is_failure_u64(), false);
    assert_eq!(command_return.is_success(), false);
    assert_eq!(command_return.is_success_u32(), false);
    assert_eq!(command_return.is_success_2_u32(), false);
    assert_eq!(command_return.is_success_u64(), false);
    assert_eq!(command_return.is_success_3_u32(), true);
    assert_eq!(command_return.is_success_u32_u64(), false);
    assert_eq!(command_return.get_failure(), None);
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), Some((1001, 1002, 1003)));
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(command_return.return_type(), return_type::SUCCESS_3_U32);
}

#[test]
fn success_u32_u64() {
    let command_return = CommandReturn {
        return_type: return_type::SUCCESS_U32_U64,
        r1: 1001,
        r2: 0x00001002,
        r3: 0x00001003,
    };
    assert_eq!(command_return.is_failure(), false);
    assert_eq!(command_return.is_failure_u32(), false);
    assert_eq!(command_return.is_failure_2_u32(), false);
    assert_eq!(command_return.is_failure_u64(), false);
    assert_eq!(command_return.is_success(), false);
    assert_eq!(command_return.is_success_u32(), false);
    assert_eq!(command_return.is_success_2_u32(), false);
    assert_eq!(command_return.is_success_u64(), false);
    assert_eq!(command_return.is_success_3_u32(), false);
    assert_eq!(command_return.is_success_u32_u64(), true);
    assert_eq!(command_return.get_failure(), None);
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(
        command_return.get_success_u32_u64(),
        Some((1001, 0x00001003_00001002))
    );
    assert_eq!(command_return.return_type(), return_type::SUCCESS_U32_U64);
}
