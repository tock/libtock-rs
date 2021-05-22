/// The return value from a yield_no_wait call.
// Calling yield-no-wait passes a *mut YieldNoWaitReturn to the kernel, which
// the kernel writes to. We cannot safely pass a `*mut bool` to the kernel,
// because the representation of `bool` in Rust is undefined (although it is
// likely false == 0, true == 1, based on `bool`'s conversions). Using *mut
// YieldNoWaitReturn rather than a *mut u8 allows the compiler to assume the
// kernel will never write a value other than 0 or 1 into the pointee. Assuming
// the likely representation of `bool`, this makes the conversion into `bool`
// free.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum YieldNoWaitReturn {
    NoUpcall = 0,
    Upcall = 1,
}
