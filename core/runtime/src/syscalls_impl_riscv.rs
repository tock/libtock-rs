use libtock_platform::{RawSyscalls, ReturnVariant};

impl RawSyscalls for crate::TockSyscalls {
    // This yield implementation is currently limited to RISC-V versions without
    // floating-point registers, as it does not mark them clobbered.
    #[cfg(not(any(target_feature = "d", target_feature = "f")))]
    fn raw_yield_no_wait(flag: &mut u8) {
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
                 inlateout("x10") 0 => _,    // a0
                 inlateout("x11") flag => _, // a1
                 lateout("x12") _,           // a2
                 lateout("x13") _,           // a3
                 inlateout("x14") 0 => _,    // a4
                 lateout("x15") _,           // a5
                 lateout("x16") _,           // a6
                 lateout("x17") _,           // a7
                 // x18-27 are s2-s11 and are callee-saved
                 lateout("x28") _, // t3
                 lateout("x29") _, // t4
                 lateout("x30") _, // t5
                 lateout("x31") _, // t6
            );
        }
    }

    // This yield implementation is currently limited to RISC-V versions without
    // floating-point registers, as it does not mark them clobbered.
    #[cfg(not(any(target_feature = "d", target_feature = "f")))]
    fn raw_yield_wait() {
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
                 inlateout("x10") 1 => _, // a0
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
    }

    unsafe fn one_arg_syscall(op: u32, class: u8) -> (ReturnVariant, usize) {
        let r0_out;
        let r1;
        asm!("ecall",
             inlateout("a0") op => r0_out,
             lateout("a1") r1,
             in("a4") class as usize, // Cast needed to zero high bits
             options(preserves_flags, nostack, nomem),
        );
        (r0_out, r1)
    }

    unsafe fn two_arg_syscall(op: u32, r1: usize, class: u8) -> (ReturnVariant, usize) {
        let r0_out;
        asm!("ecall",
             inlateout("a0") op => r0_out,
             inlateout("a1") r1,
             in("a4") class as usize, // Cast needed to zero high bits
             options(preserves_flags, nostack, nomem)
        );
        (r0_out, r1)
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
             in("a4") class as usize, // Cast needed to zero high bits
             options(preserves_flags, nostack),
        );
        (r0, r1 as usize, r2, r3)
    }
}
