// Contains various types used by the `Syscalls` trait. These are in a separate
// file from `Syscalls` to keep the size of syscalls.rs reasonable.

#[non_exhaustive]
#[repr(usize)]
pub enum OneArgMemop {
    Brk = 0,
    Sbrk = 1,
    FlashRegionStart = 8,
    FlashRegionEnd = 9,
    SpecifyStackTop = 10,
    SpecifyHeapStart = 11,
}

pub enum ReturnType {
    Failure = 0,
    FailureWithU32 = 1,
    FailureWith2U32 = 2,
    FailureWithU64 = 3,
    Success = 128,
    SuccessWithU32 = 129,
    SuccessWith2U32 = 130,
    SuccessWithU64 = 131,
    SuccessWith3U32 = 132,
    SuccessWithU32AndU64 = 133,
}

// TODO: When the numeric values (0 and 1) are assigned to the yield types,
// specify those values here.
#[non_exhaustive]
#[repr(usize)]
pub enum YieldType {
    Wait,
    NoWait,
}

#[non_exhaustive]
#[repr(usize)]
pub enum ZeroArgMemop {
    MemoryStart = 2,
    MemoryEnd = 3,
    FlashStart = 4,
    FlashEnd = 5,
    GrantStart = 6,
    FlashRegions = 7,
}
