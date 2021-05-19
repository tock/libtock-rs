//! Safe constructors for `libtock_platform::CommandReturn` variants.

use libtock_platform::{return_variant, CommandReturn, ErrorCode};

pub fn failure(error_code: ErrorCode) -> CommandReturn {
    // Safety: return_variant is a failure, so r1 must be a valid ErrorCode,
    // which is enforced by error_code's type.
    unsafe { CommandReturn::new(return_variant::FAILURE, error_code as u32, 0, 0) }
}

pub fn failure_u32(error_code: ErrorCode, value: u32) -> CommandReturn {
    // Safety: return_variant is a failure, so r1 must be a valid ErrorCode,
    // which is enforced by error_code's type.
    unsafe { CommandReturn::new(return_variant::FAILURE_U32, error_code as u32, value, 0) }
}

pub fn failure_2_u32(error_code: ErrorCode, value0: u32, value1: u32) -> CommandReturn {
    unsafe {
        // Safety: return_variant is a failure, so r1 must be a valid ErrorCode,
        // which is enforced by error_code's type.
        CommandReturn::new(
            return_variant::FAILURE_2_U32,
            error_code as u32,
            value0,
            value1,
        )
    }
}

pub fn failure_u64(error_code: ErrorCode, value: u64) -> CommandReturn {
    unsafe {
        // Safety: return_variant is a failure, so r1 must be a valid ErrorCode,
        // which is enforced by error_code's type.
        CommandReturn::new(
            return_variant::FAILURE_U64,
            error_code as u32,
            value as u32,
            (value >> 32) as u32,
        )
    }
}

pub fn success() -> CommandReturn {
    // Safety: return_variant is a success so there are no other invariants to
    // maintain.
    unsafe { CommandReturn::new(return_variant::SUCCESS, 0, 0, 0) }
}

pub fn success_u32(value: u32) -> CommandReturn {
    // Safety: return_variant is a success so there are no other invariants to
    // maintain.
    unsafe { CommandReturn::new(return_variant::SUCCESS_U32, value, 0, 0) }
}

pub fn success_2_u32(value0: u32, value1: u32) -> CommandReturn {
    // Safety: return_variant is a success so there are no other invariants to
    // maintain.
    unsafe { CommandReturn::new(return_variant::SUCCESS_2_U32, value0, value1, 0) }
}

pub fn success_u64(value: u64) -> CommandReturn {
    unsafe {
        // Safety: return_variant is a success so there are no other invariants
        // to maintain.
        CommandReturn::new(
            return_variant::SUCCESS_U64,
            value as u32,
            (value >> 32) as u32,
            0,
        )
    }
}

pub fn success_3_u32(value0: u32, value1: u32, value2: u32) -> CommandReturn {
    // Safety: return_variant is a success so there are no other invariants to
    // maintain.
    unsafe { CommandReturn::new(return_variant::SUCCESS_3_U32, value0, value1, value2) }
}

pub fn success_u32_u64(value0: u32, value1: u64) -> CommandReturn {
    unsafe {
        // Safety: return_variant is a success so there are no other invariants
        // to maintain.
        CommandReturn::new(
            return_variant::SUCCESS_U32_U64,
            value0,
            value1 as u32,
            (value1 >> 32) as u32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_test() {
        assert_eq!(
            failure(ErrorCode::Fail).get_failure(),
            Some(ErrorCode::Fail)
        );
    }

    #[test]
    fn failure_u32_test() {
        assert_eq!(
            failure_u32(ErrorCode::Busy, 42).get_failure_u32(),
            Some((ErrorCode::Busy, 42))
        );
    }

    #[test]
    fn failure_2_u32_test() {
        assert_eq!(
            failure_2_u32(ErrorCode::Off, 31, 27).get_failure_2_u32(),
            Some((ErrorCode::Off, 31, 27))
        );
    }

    #[test]
    fn failure_u64_test() {
        assert_eq!(
            failure_u64(ErrorCode::Size, 0x1111_2222_3333_4444).get_failure_u64(),
            Some((ErrorCode::Size, 0x1111_2222_3333_4444))
        );
    }

    #[test]
    fn success_test() {
        assert!(success().is_success());
    }

    #[test]
    fn success_u32_test() {
        assert_eq!(success_u32(1618).get_success_u32(), Some(1618));
    }

    #[test]
    fn success_2_u32_test() {
        assert_eq!(success_2_u32(1, 2).get_success_2_u32(), Some((1, 2)));
    }

    #[test]
    fn success_u64_test() {
        assert_eq!(
            success_u64(0x1111_2222_3333_4444).get_success_u64(),
            Some(0x1111_2222_3333_4444)
        );
    }

    #[test]
    fn success_3_u32_test() {
        assert_eq!(success_3_u32(3, 5, 8).get_success_3_u32(), Some((3, 5, 8)));
    }

    #[test]
    fn success_u32_u64_test() {
        assert_eq!(
            success_u32_u64(13, 0x1111_2222_3333_4444).get_success_u32_u64(),
            Some((13, 0x1111_2222_3333_4444))
        );
    }
}
