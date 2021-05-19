use crate::{return_variant, CommandReturn, ErrorCode};

#[test]
fn failure() {
    let command_return = unsafe {
        CommandReturn::new(
            return_variant::FAILURE,
            ErrorCode::Reserve as u32,
            1002,
            1003,
        )
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
    assert_eq!(command_return.get_failure(), Some(ErrorCode::Reserve));
    assert_eq!(command_return.get_failure_u32(), None);
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(
        command_return.raw_values(),
        (return_variant::FAILURE, 5, 1002, 1003)
    );
    assert_eq!(command_return.return_variant(), return_variant::FAILURE);
}

#[test]
fn failure_u32() {
    let command_return = unsafe {
        CommandReturn::new(
            return_variant::FAILURE_U32,
            ErrorCode::Off as u32,
            1002,
            1003,
        )
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
        Some((ErrorCode::Off, 1002))
    );
    assert_eq!(command_return.get_failure_2_u32(), None);
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(
        command_return.raw_values(),
        (return_variant::FAILURE_U32, 4, 1002, 1003)
    );
    assert_eq!(command_return.return_variant(), return_variant::FAILURE_U32);
}

#[test]
fn failure_2_u32() {
    let command_return = unsafe {
        CommandReturn::new(
            return_variant::FAILURE_2_U32,
            ErrorCode::Already as u32,
            1002,
            1003,
        )
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
        Some((ErrorCode::Already, 1002, 1003))
    );
    assert_eq!(command_return.get_failure_u64(), None);
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(
        command_return.raw_values(),
        (return_variant::FAILURE_2_U32, 3, 1002, 1003)
    );
    assert_eq!(
        command_return.return_variant(),
        return_variant::FAILURE_2_U32
    );
}

#[test]
fn failure_u64() {
    let command_return = unsafe {
        CommandReturn::new(
            return_variant::FAILURE_U64,
            ErrorCode::Busy as u32,
            0x1002,
            0x1003,
        )
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
        Some((ErrorCode::Busy, 0x0000_1003_0000_1002))
    );
    assert_eq!(command_return.get_success_u32(), None);
    assert_eq!(command_return.get_success_2_u32(), None);
    assert_eq!(command_return.get_success_u64(), None);
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(
        command_return.raw_values(),
        (return_variant::FAILURE_U64, 2, 0x1002, 0x1003)
    );
    assert_eq!(command_return.return_variant(), return_variant::FAILURE_U64);
}

#[test]
fn success() {
    let command_return = unsafe { CommandReturn::new(return_variant::SUCCESS, 1001, 1002, 1003) };
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
    assert_eq!(
        command_return.raw_values(),
        (return_variant::SUCCESS, 1001, 1002, 1003)
    );
    assert_eq!(command_return.return_variant(), return_variant::SUCCESS);
}

#[test]
fn success_u32() {
    let command_return =
        unsafe { CommandReturn::new(return_variant::SUCCESS_U32, 1001, 1002, 1003) };
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
    assert_eq!(
        command_return.raw_values(),
        (return_variant::SUCCESS_U32, 1001, 1002, 1003)
    );
    assert_eq!(command_return.return_variant(), return_variant::SUCCESS_U32);
}

#[test]
fn success_2_u32() {
    let command_return =
        unsafe { CommandReturn::new(return_variant::SUCCESS_2_U32, 1001, 1002, 1003) };
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
    assert_eq!(
        command_return.raw_values(),
        (return_variant::SUCCESS_2_U32, 1001, 1002, 1003)
    );
    assert_eq!(
        command_return.return_variant(),
        return_variant::SUCCESS_2_U32
    );
}

#[test]
fn success_u64() {
    let command_return =
        unsafe { CommandReturn::new(return_variant::SUCCESS_U64, 0x1001, 0x1002, 1003) };
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
    assert_eq!(
        command_return.get_success_u64(),
        Some(0x0000_1002_0000_1001)
    );
    assert_eq!(command_return.get_success_3_u32(), None);
    assert_eq!(command_return.get_success_u32_u64(), None);
    assert_eq!(
        command_return.raw_values(),
        (return_variant::SUCCESS_U64, 0x1001, 0x1002, 1003)
    );
    assert_eq!(command_return.return_variant(), return_variant::SUCCESS_U64);
}

#[test]
fn success_3_u32() {
    let command_return =
        unsafe { CommandReturn::new(return_variant::SUCCESS_3_U32, 1001, 1002, 1003) };
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
    assert_eq!(
        command_return.raw_values(),
        (return_variant::SUCCESS_3_U32, 1001, 1002, 1003)
    );
    assert_eq!(
        command_return.return_variant(),
        return_variant::SUCCESS_3_U32
    );
}

#[test]
fn success_u32_u64() {
    let command_return =
        unsafe { CommandReturn::new(return_variant::SUCCESS_U32_U64, 1001, 0x1002, 0x1003) };
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
        Some((1001, 0x0000_1003_0000_1002))
    );
    assert_eq!(
        command_return.raw_values(),
        (return_variant::SUCCESS_U32_U64, 1001, 0x1002, 0x1003)
    );
    assert_eq!(
        command_return.return_variant(),
        return_variant::SUCCESS_U32_U64
    );
}
