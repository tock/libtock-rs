use libtock_platform::{OneArgMemop, RawSyscalls, YieldType, ZeroArgMemop};

impl RawSyscalls for crate::TockSyscalls {
    // This yield implementation is currently limited RISC-V versions without
    // floating-point registers, as it does not mark them clobbered.
    #[cfg(not(any(target_feature = "d", target_feature = "f")))]
    fn raw_yield(r0_in: YieldType) -> u32 {
        let mut r0 = r0_in as u32;
        unsafe {
            asm!("ecall",
                 // x0 is the zero register.
                 lateout("x1") _, // Return address
                 // x2-x4 are stack, global, and thread pointers. sp is
                 // callee-saved.
                 lateout("x5") _, // t0
                 lateout("x6") _, // t1
                 lateout("x7") _, // t2
                 // x8 and x9 are s0 and s1 and are callee-saved.
                 inlateout("x10") r0,     // a0
                 lateout("x11") _,        // a1
                 lateout("x12") _,        // a2
                 lateout("x13") _,        // a3
                 inlateout("x14") 0 => _, // a4
                 lateout("x15") _,        // a5
                 lateout("x16") _,        // a6
                 lateout("x17") _,        // a7
                 // x18-27 are s2-s11 and are callee-saved
                 lateout("x28") _, // t3
                 lateout("x29") _, // t4
                 lateout("x30") _, // t5
                 lateout("x31") _, // t6
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
