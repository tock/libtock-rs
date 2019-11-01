use crate::callback::CallbackSubscription;
use crate::callback::SubscribableCallback;
use crate::shared_memory::SharedMemory;

#[cfg(target_arch = "arm")]
pub fn yieldk() {
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
    unsafe {
        asm!(
            "svc 0"
            :
            :
            : "memory", "r0", "r1", "r2", "r3", "r12", "lr"
            : "volatile");
    }
}

#[cfg(target_arch = "riscv32")]
pub fn yieldk() {
    /* TODO: Stop yielding */
    unsafe {
        asm! (
            "li    a0, 0
            ecall"
            :
            :
            : "memory", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7",
            "t0", "t1", "t2", "t3", "t4", "t5", "t6", "ra"
            : "volatile");
    }
}

pub fn yieldk_for<F: Fn() -> bool>(cond: F) {
    while !cond() {
        yieldk();
    }
}

pub fn subscribe<CB: SubscribableCallback>(
    driver_number: usize,
    subscribe_number: usize,
    callback: &mut CB,
) -> Result<CallbackSubscription, isize> {
    let return_code = unsafe {
        subscribe_ptr(
            driver_number,
            subscribe_number,
            c_callback::<CB> as *const _,
            callback as *mut CB as usize,
        )
    };

    if return_code == 0 {
        Ok(CallbackSubscription::new(driver_number, subscribe_number))
    } else {
        Err(return_code)
    }
}

extern "C" fn c_callback<CB: SubscribableCallback>(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    userdata: usize,
) {
    let callback = unsafe { &mut *(userdata as *mut CB) };
    callback.call_rust(arg0, arg1, arg2);
}

#[cfg(target_arch = "arm")]
pub unsafe fn subscribe_ptr(
    major: usize,
    minor: usize,
    cb: *const unsafe extern "C" fn(usize, usize, usize, usize),
    ud: usize,
) -> isize {
    let res;
    asm!("svc 1" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(cb) "{r3}"(ud)
                 : "memory"
                 : "volatile");
    res
}

#[cfg(target_arch = "riscv32")]
pub unsafe fn subscribe_ptr(
    major: usize,
    minor: usize,
    cb: *const unsafe extern "C" fn(usize, usize, usize, usize),
    ud: usize,
) -> isize {
    let res;
    asm!("li    a0, 1
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (cb), "{x14}" (ud)
         : "memory"
         : "volatile" );
    res
}

#[cfg(target_arch = "arm")]
#[inline(always)]
pub unsafe fn command(major: usize, minor: usize, arg1: usize, arg2: usize) -> isize {
    let res;
    asm!("svc 2" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(arg1) "{r3}"(arg2)
                 : "memory"
                 : "volatile");
    res
}

#[cfg(target_arch = "riscv32")]
#[inline(always)]
pub unsafe fn command(major: usize, minor: usize, arg1: usize, arg2: usize) -> isize {
    let res;
    asm!("li    a0, 2
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (arg1), "{x14}" (arg2)
         : "memory"
         : "volatile");
    res
}

// command1_insecure, is a variant of command() that only sets the first
// argument in the system call interface. It has the benefit of generating
// simpler assembly than command(), but it leaves the second argument's register
// as-is which leaks it to the kernel driver being called. Prefer to use
// command() instead of command1_insecure(), unless the benefit of generating
// simpler assembly outweighs the drawbacks of potentially leaking arbitrary
// information to the driver you are calling.
//
// At the moment, the only suitable use case for command1_insecure is the low
// level debug interface.

#[cfg(target_arch = "arm")]
#[inline(always)]
pub unsafe fn command1_insecure(major: usize, minor: usize, arg: usize) -> isize {
    let res;
    asm!("svc 2" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(arg)
                 : "memory"
                 : "volatile");
    res
}

#[cfg(target_arch = "riscv32")]
#[inline(always)]
pub unsafe fn command1_insecure(major: usize, minor: usize, arg: usize) -> isize {
    let res;
    asm!("li    a0, 2
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (arg)
         : "memory"
         : "volatile");
    res
}

pub fn allow(
    driver_number: usize,
    allow_number: usize,
    buffer_to_share: &mut [u8],
) -> Result<SharedMemory, isize> {
    let len = buffer_to_share.len();
    let return_code = unsafe {
        allow_ptr(
            driver_number,
            allow_number,
            buffer_to_share.as_mut_ptr(),
            len,
        )
    };
    if return_code == 0 {
        Ok(SharedMemory::new(
            driver_number,
            allow_number,
            buffer_to_share,
        ))
    } else {
        Err(return_code)
    }
}

#[cfg(target_arch = "arm")]
pub unsafe fn allow_ptr(major: usize, minor: usize, slice: *mut u8, len: usize) -> isize {
    let res;
    asm!("svc 3" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(slice) "{r3}"(len)
                 : "memory"
                 : "volatile");
    res
}

#[cfg(target_arch = "riscv32")]
pub unsafe fn allow_ptr(major: usize, minor: usize, slice: *mut u8, len: usize) -> isize {
    let res;
    asm!("li    a0, 3
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (minor), "{x13}" (slice), "{x14}" (len)
         : "memory"
         : "volatile");
    res
}

#[cfg(target_arch = "arm")]
pub unsafe fn memop(major: u32, arg1: usize) -> isize {
    let res;
    asm!("svc 4" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(arg1)
                 : "memory"
                 : "volatile");
    res
}

#[cfg(target_arch = "riscv32")]
pub unsafe fn memop(major: u32, arg1: usize) -> isize {
    let res;
    asm!("li    a0, 4
          ecall"
         : "={x10}" (res)
         : "{x11}" (major), "{x12}" (arg1)
         : "memory"
         : "volatile");
    res
}
