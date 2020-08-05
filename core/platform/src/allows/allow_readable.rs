/// Because the kernel may modify shared memory and place any bit pattern into
/// that memory, we can only read types out of shared memory if every bit
/// pattern is a valid value of that type. `AllowReadable` types are safe to
/// read out of a shared buffer.
pub unsafe trait AllowReadable {}

unsafe impl AllowReadable for i8 {}
unsafe impl AllowReadable for i16 {}
unsafe impl AllowReadable for i32 {}
unsafe impl AllowReadable for i64 {}
unsafe impl AllowReadable for i128 {}
unsafe impl AllowReadable for isize {}
unsafe impl AllowReadable for u8 {}
unsafe impl AllowReadable for u16 {}
unsafe impl AllowReadable for u32 {}
unsafe impl AllowReadable for u64 {}
unsafe impl AllowReadable for u128 {}
unsafe impl AllowReadable for usize {}
