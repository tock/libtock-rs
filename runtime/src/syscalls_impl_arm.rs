use libtock_platform::{RawSyscalls, Register};

unsafe impl RawSyscalls for crate::TockSyscalls {
    unsafe fn yield1([Register(r0)]: [Register; 1]) {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::yield1
        unsafe {
            asm!("svc 0",
                 inlateout("r0") r0 => _, // a1
                 lateout("r1") _,         // a2
                 lateout("r2") _,         // a3
                 lateout("r3") _,         // a4
                 // r4-r8 are callee-saved.
                 // r9 is platform-specific. We don't use it in libtock_runtime,
                 // so it is either unused or used as a callee-saved register.
                 // r10 and r11 are callee-saved.
                 lateout("r12") _, // ip
                 // r13 is the stack pointer and must be restored by the callee.
                 lateout("r14") _, // lr
                 // r15 is the program counter.
            );
        }
    }

    unsafe fn yield2([Register(r0), Register(r1)]: [Register; 2]) {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::yield2
        unsafe {
            asm!("svc 0",
                 inlateout("r0") r0 => _, // a1
                 inlateout("r1") r1 => _, // a2
                 lateout("r2") _,         // a3
                 lateout("r3") _,         // a4
                 // r4-r8 are callee-saved.
                 // r9 is platform-specific. We don't use it in libtock_runtime,
                 // so it is either unused or used as a callee-saved register.
                 // r10 and r11 are callee-saved.
                 lateout("r12") _, // ip
                 // r13 is the stack pointer and must be restored by the callee.
                 lateout("r14") _, // lr
                 // r15 is the program counter.
            );
        }
    }

    unsafe fn syscall1<const CLASS: usize>([Register(mut r0)]: [Register; 1]) -> [Register; 2] {
        let r1;
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::syscall1
        unsafe {
            asm!("svc {}",
                 const CLASS,
                 inlateout("r0") r0,
                 lateout("r1") r1,
                 options(preserves_flags, nostack, nomem),
            );
        }
        [Register(r0), Register(r1)]
    }

    unsafe fn syscall2<const CLASS: usize>(
        [Register(mut r0), Register(mut r1)]: [Register; 2],
    ) -> [Register; 2] {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::syscall2
        unsafe {
            asm!("svc {}",
                 const CLASS,
                 inlateout("r0") r0,
                 inlateout("r1") r1,
                 options(preserves_flags, nostack, nomem)
            );
        }
        [Register(r0), Register(r1)]
    }

    unsafe fn syscall4<const CLASS: usize>(
        [Register(mut r0), Register(mut r1), Register(mut r2), Register(mut r3)]: [Register; 4],
    ) -> [Register; 4] {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::syscall4
        unsafe {
            asm!("svc {}",
                 const CLASS,
                 inlateout("r0") r0,
                 inlateout("r1") r1,
                 inlateout("r2") r2,
                 inlateout("r3") r3,
                 options(preserves_flags, nostack),
            );
        }
        [Register(r0), Register(r1), Register(r2), Register(r3)]
    }
}
