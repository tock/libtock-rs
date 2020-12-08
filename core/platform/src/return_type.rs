/// `ReturnType` describes what value type the kernel has returned.
// ReturnType is not an enum so that it can be converted from a u32 for free.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ReturnType(u32);

impl From<u32> for ReturnType {
    fn from(value: u32) -> ReturnType {
        ReturnType(value)
    }
}

impl From<ReturnType> for u32 {
    fn from(return_type: ReturnType) -> u32 {
        return_type.0
    }
}

pub const FAILURE: ReturnType = ReturnType(0);
pub const FAILURE_U32: ReturnType = ReturnType(1);
pub const FAILURE_2_U32: ReturnType = ReturnType(2);
pub const FAILURE_U64: ReturnType = ReturnType(3);
pub const SUCCESS: ReturnType = ReturnType(128);
pub const SUCCESS_U32: ReturnType = ReturnType(129);
pub const SUCCESS_2_U32: ReturnType = ReturnType(130);
pub const SUCCESS_U64: ReturnType = ReturnType(131);
pub const SUCCESS_3_U32: ReturnType = ReturnType(132);
pub const SUCCESS_U32_U64: ReturnType = ReturnType(133);
