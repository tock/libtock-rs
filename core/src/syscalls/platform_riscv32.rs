#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn yieldk() {
    /* TODO: Stop yielding */
    llvm_asm! (
            "li    a0, 0
            ecall"
            :
            :
            : "memory", "x10", "x11", "x12", "x13", "x14", "x15", "x16", "x17",
            "x5", "x6", "x7", "x28", "x29", "x30", "x31", "x1"
            : "volatile");
}

#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn subscribe(
    major: usize,
    minor: usize,
    cb: *const unsafe extern "C" fn(usize, usize, usize, usize),
    ud: usize,
) -> isize {
    let res;
    llvm_asm!("li    a0, 1
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (cb), "{x14}" (ud)
         : "memory"
         : "volatile" );
    res
}

#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn command(major: usize, minor: usize, arg1: usize, arg2: usize) -> isize {
    let res;
    llvm_asm!("li    a0, 2
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (arg1), "{x14}" (arg2)
         : "memory"
         : "volatile");
    res
}

#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn command1(major: usize, minor: usize, arg: usize) -> isize {
    let res;
    llvm_asm!("li    a0, 2
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (arg)
         : "memory"
         : "volatile");
    res
}

#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn allow(major: usize, minor: usize, slice: *mut u8, len: usize) -> isize {
    let res;
    llvm_asm!("li    a0, 3
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (slice), "{x14}" (len)
         : "memory"
         : "volatile");
    res
}

#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn memop(major: u32, arg1: usize) -> isize {
    let res;
    llvm_asm!("li    a0, 4
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (arg1)
         : "memory"
         : "volatile");
    res
}
