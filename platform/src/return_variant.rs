// TODO: Re-evaluate whether u32 is the correct type to wrap. Maybe it should
// wrap a Register instead? Also, double-check that ReturnVariant is providing
// useful type-safety.

/// `ReturnVariant` describes what value type the kernel has returned.
// ReturnVariant is not an enum so that it can be converted from a u32 for free.
// TODO: Add a ufmt debug implementation for use by process binaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReturnVariant(u32);

impl From<u32> for ReturnVariant {
    fn from(value: u32) -> ReturnVariant {
        ReturnVariant(value)
    }
}

impl From<ReturnVariant> for crate::Register {
    fn from(return_variant: ReturnVariant) -> crate::Register {
        return_variant.0.into()
    }
}

impl From<ReturnVariant> for u32 {
    fn from(return_variant: ReturnVariant) -> u32 {
        return_variant.0
    }
}

pub const FAILURE: ReturnVariant = ReturnVariant(0);
pub const FAILURE_U32: ReturnVariant = ReturnVariant(1);
pub const FAILURE_2_U32: ReturnVariant = ReturnVariant(2);
pub const FAILURE_U64: ReturnVariant = ReturnVariant(3);
pub const SUCCESS: ReturnVariant = ReturnVariant(128);
pub const SUCCESS_U32: ReturnVariant = ReturnVariant(129);
pub const SUCCESS_2_U32: ReturnVariant = ReturnVariant(130);
pub const SUCCESS_U64: ReturnVariant = ReturnVariant(131);
pub const SUCCESS_3_U32: ReturnVariant = ReturnVariant(132);
pub const SUCCESS_U32_U64: ReturnVariant = ReturnVariant(133);
