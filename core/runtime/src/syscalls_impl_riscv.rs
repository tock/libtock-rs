use libtock_platform::{OneArgMemop, RawSyscalls, YieldType, ZeroArgMemop};

impl RawSyscalls for crate::TockSyscalls {
    // This yield implementation is currently limited RISC-V versions without
    // floating-point registers, as it does not mark them clobbered.
    #[cfg(not(any(target_feature = "d", target_feature = "f")))]
    fn raw_yield(r0_in: YieldType) -> u32 {
        let mut r0 = r0_in as u32;
        let mut _a4 = 0;
        unsafe {
            asm!("ecall",
                 // x0 is a constant.
                 lateout("x1") _, // Return address
                 // x2-x4 are stack, global, and thread pointers. sp is
                 // callee-saved.
                 lateout("x5") _,
                 lateout("x6") _,
                 lateout("x7") _,
                 // x8 and x9 are callee-saved.
                 inlateout("x10") r0,
                 lateout("x11") _,
                 lateout("x12") _,
                 lateout("x13") _,
                 inlateout("x14") _a4,
                 lateout("x15") _,
                 lateout("x16") _,
                 lateout("x17") _,
                 // x18-27 (aka s2-s11) are callee-saved
                 lateout("x28") _,
                 lateout("x29") _,
                 lateout("x30") _,
                 lateout("x31") _,
            );
        }
        r0
    }

    unsafe fn four_arg_syscall(
        mut r0: u32,
        mut r1: u32,
        mut r2: usize,
        mut r3: usize,
        class: u8,
    ) -> (u32, usize, usize, usize) {
        asm!("ecall",
             inlateout("a0") r0,
             inlateout("a1") r1,
             inlateout("a2") r2,
             inlateout("a3") r3,
             in("a4") class,
             options(preserves_flags, nostack),
        );
        (r0, r1 as usize, r2, r3)
    }

    fn zero_arg_memop(r0_in: ZeroArgMemop) -> (u32, usize) {
        let mut r0 = r0_in as u32;
        let r1;
        unsafe {
            asm!("ecall",
                 inlateout("a0") r0,
                 lateout("a1") r1,
                 in("a4") 5,
                 options(preserves_flags, nostack, nomem),
            );
        }
        (r0, r1)
    }

    fn one_arg_memop(r0_in: OneArgMemop, mut r1: usize) -> (u32, usize) {
        let mut r0 = r0_in as u32;
        unsafe {
            asm!("ecall",
                 inlateout("a0") r0,
                 inlateout("a1") r1,
                 in("a4") 5,
                 options(preserves_flags, nostack, nomem)
            );
        }
        (r0, r1)
    }
}
