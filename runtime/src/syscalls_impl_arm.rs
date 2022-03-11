use core::arch::asm;
use libtock_platform::{syscall_class, RawSyscalls, Register};

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

    unsafe fn syscall1<const CLASS: usize>([Register(mut r0)]: [Register; 1]) -> [Register; 2] {
        let r1;
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::syscall1
        unsafe {
            // Syscall class 5 is Memop, the only syscall class that syscall1
            // supports.
            asm!("svc 5",
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
            // TODO: Replace this match statement with a `const` operand when
            // asm_const [1] is stabilized, or redesign RawSyscalls to not need
            // this match statement.
            //
            // [1] https://github.com/rust-lang/rust/issues/93332
            match CLASS {
                syscall_class::MEMOP => asm!("svc 5",
                     inlateout("r0") r0,
                     inlateout("r1") r1,
                     options(preserves_flags, nostack, nomem)
                ),
                syscall_class::EXIT => asm!("svc 6",
                     inlateout("r0") r0,
                     inlateout("r1") r1,
                     options(preserves_flags, nostack, nomem)
                ),
                _ => unreachable!(),
            }
        }
        [Register(r0), Register(r1)]
    }

    unsafe fn syscall4<const CLASS: usize>(
        [Register(mut r0), Register(mut r1), Register(mut r2), Register(mut r3)]: [Register; 4],
    ) -> [Register; 4] {
        // Safety: This matches the invariants required by the documentation on
        // RawSyscalls::syscall4
        unsafe {
            // TODO: Replace this match statement with a `const` operand when
            // asm_const [1] is stabilized, or redesign RawSyscalls to not need
            // this match statement.
            //
            // [1] https://github.com/rust-lang/rust/issues/93332
            match CLASS {
                syscall_class::SUBSCRIBE => asm!("svc 1",
                     inlateout("r0") r0,
                     inlateout("r1") r1,
                     inlateout("r2") r2,
                     inlateout("r3") r3,
                     options(preserves_flags, nostack),
                ),
                syscall_class::COMMAND => asm!("svc 2",
                     inlateout("r0") r0,
                     inlateout("r1") r1,
                     inlateout("r2") r2,
                     inlateout("r3") r3,
                     options(preserves_flags, nostack),
                ),
                syscall_class::ALLOW_RW => asm!("svc 3",
                     inlateout("r0") r0,
                     inlateout("r1") r1,
                     inlateout("r2") r2,
                     inlateout("r3") r3,
                     options(preserves_flags, nostack),
                ),
                syscall_class::ALLOW_RO => asm!("svc 4",
                     inlateout("r0") r0,
                     inlateout("r1") r1,
                     inlateout("r2") r2,
                     inlateout("r3") r3,
                     options(preserves_flags, nostack),
                ),
                _ => unreachable!(),
            }
        }
        [Register(r0), Register(r1), Register(r2), Register(r3)]
    }
}
