#[inline(always)]
pub unsafe fn yieldk() {
    // Note: A process stops yielding when there is a callback ready to run,
    // which the kernel executes by modifying the stack frame pushed by the
    // hardware. The kernel copies the PC value from the stack frame to the LR
    // field, and sets the PC value to callback to run. When this frame is
    // unstacked during the interrupt return, the effectively clobbers the LR
    // register.
    //
    // At this point, the callback function is now executing, which may itself
    // clobber any of the other caller-saved registers. Thus we mark this
    // inline assembly as conservatively clobbering all caller-saved registers,
    // forcing yield to save any live registers.
    //
    // Upon direct observation of this function, the LR is the only register
    // that is live across the SVC invocation, however, if the yield call is
    // inlined, it is possible that the LR won't be live at all (commonly seen
    // for the `loop { yieldk(); }` idiom) or that other registers are live,
    // thus it is important to let the compiler do the work here.
    //
    // According to the AAPCS: A subroutine must preserve the contents of the
    // registers r4-r8, r10, r11 and SP (and r9 in PCS variants that designate
    // r9 as v6) As our compilation flags mark r9 as the PIC base register, it
    // does not need to be saved. Thus we must clobber r0-3, r12, and LR
    llvm_asm!(
            "svc 0"
            :
            :
            : "memory", "r0", "r1", "r2", "r3", "r12", "lr"
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
    llvm_asm!("svc 1" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(cb) "{r3}"(ud)
                 : "memory"
                 : "volatile");
    res
}

#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn command(major: usize, minor: usize, arg1: usize, arg2: usize) -> isize {
    let res;
    llvm_asm!("svc 2" : "={r0}"(res)
                     : "{r0}"(major) "{r1}"(minor) "{r2}"(arg1) "{r3}"(arg2)
                     : "memory"
                     : "volatile");
    res
}

#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn command1(major: usize, minor: usize, arg: usize) -> isize {
    let res;
    llvm_asm!("svc 2" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(arg)
                 : "memory"
                 : "volatile");
    res
}

#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn allow(major: usize, minor: usize, slice: *mut u8, len: usize) -> isize {
    let res;
    llvm_asm!("svc 3" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(slice) "{r3}"(len)
                 : "memory"
                 : "volatile");
    res
}

#[inline(always)]
// Justification: documentation is generated from mocks
#[allow(clippy::missing_safety_doc)]
pub unsafe fn memop(major: u32, arg1: usize) -> isize {
    let res;
    llvm_asm!("svc 4" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(arg1)
                 : "memory"
                 : "volatile");
    res
}
