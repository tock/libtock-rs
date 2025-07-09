use core::arch::asm;
use libtock_platform::{RawSyscalls, Register};

unsafe impl RawSyscalls for crate::TockSyscalls {
    unsafe fn yield1([Register(r0)]: [Register; 1]) {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::yield1
        // the use of `clobber_abi` allows us this to run on both Thumb-1 and Thumb-2
        unsafe {
            asm!("svc 0",
                 inlateout("r0") r0 => _, // a1
                 // r4-r8 are callee-saved.
                 // r9 is platform-specific. We don't use it in libtock_runtime,
                 // so it is either unused or used as a callee-saved register.
                 // r10 and r11 are callee-saved.

                 // r13 is the stack pointer and must be restored by the callee.
                 // r15 is the program counter.

                 clobber_abi("C"), // a2, a3, a4, ip (r12), lr (r14)
            );
        }
    }

    unsafe fn yield2([Register(r0), Register(r1)]: [Register; 2]) {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::yield2
        // the use of `clobber_abi` allows us this to run on both Thumb-1 and Thumb-2
        unsafe {
            asm!("svc 0",
                 inlateout("r0") r0 => _, // a1
                 inlateout("r1") r1 => _, // a2
                 // r4-r8 are callee-saved.
                 // r9 is platform-specific. We don't use it in libtock_runtime,
                 // so it is either unused or used as a callee-saved register.
                 // r10 and r11 are callee-saved.

                 // r13 is the stack pointer and must be restored by the callee.
                 // r15 is the program counter.

                 clobber_abi("C"), // a3, a4, ip (r12), lr (r14)
            );
        }
    }

    unsafe fn yield3([Register(r0), Register(r1), Register(r2)]: [Register; 3]) {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::yield2
        // the use of `clobber_abi` allows us this to run on both Thumb-1 and Thumb-2
        unsafe {
            asm!("svc 0",
                 inlateout("r0") r0 => _, // a1
                 inlateout("r1") r1 => _, // a2
                 inlateout("r2") r2 => _,
                 // r4-r8 are callee-saved.
                 // r9 is platform-specific. We don't use it in libtock_runtime,
                 // so it is either unused or used as a callee-saved register.
                 // r10 and r11 are callee-saved.

                 // r13 is the stack pointer and must be restored by the callee.
                 // r15 is the program counter.

                 clobber_abi("C"), // a3, a4, ip (r12), lr (r14)
            );
        }
    }

    unsafe fn syscall1<const SYSCALL_CLASS_NUMBER: usize>(
        [Register(mut r0)]: [Register; 1],
    ) -> [Register; 2] {
        let r1;
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::syscall1
        #[allow(clippy::pointers_in_nomem_asm_block)]
        unsafe {
            asm!(
                "svc {SYSCALL_CLASS_NUMBER}",
                inlateout("r0") r0,
                lateout("r1") r1,
                options(preserves_flags, nostack, nomem),
                SYSCALL_CLASS_NUMBER = const SYSCALL_CLASS_NUMBER,
            );
        }
        [Register(r0), Register(r1)]
    }

    unsafe fn syscall2<const SYSCALL_CLASS_NUMBER: usize>(
        [Register(mut r0), Register(mut r1)]: [Register; 2],
    ) -> [Register; 2] {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::syscall2
        #[allow(clippy::pointers_in_nomem_asm_block)]
        unsafe {
            asm!(
                "svc {SYSCALL_CLASS_NUMBER}",
                 inlateout("r0") r0,
                 inlateout("r1") r1,
                 options(preserves_flags, nostack, nomem),
                 SYSCALL_CLASS_NUMBER = const SYSCALL_CLASS_NUMBER,
            );
        }
        [Register(r0), Register(r1)]
    }

    unsafe fn syscall4<const SYSCALL_CLASS_NUMBER: usize>(
        [Register(mut r0), Register(mut r1), Register(mut r2), Register(mut r3)]: [Register; 4],
    ) -> [Register; 4] {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::syscall4
        unsafe {
            asm!(
                "svc {SYSCALL_CLASS_NUMBER}",
                inlateout("r0") r0,
                inlateout("r1") r1,
                inlateout("r2") r2,
                inlateout("r3") r3,
                options(preserves_flags, nostack),
                SYSCALL_CLASS_NUMBER = const SYSCALL_CLASS_NUMBER,
            );
        }
        [Register(r0), Register(r1), Register(r2), Register(r3)]
    }
}
