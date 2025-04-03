use core::arch::asm;
use libtock_platform::{syscall_class, RawSyscalls, Register};

unsafe impl RawSyscalls for crate::TockSyscalls {
    // Yield 1 is used for yield_wait
    unsafe fn yield1([Register(r0)]: [Register; 1]) {
        unsafe {
            asm!(
                "pushl $0",
                "pushl $0",
                "pushl $0",
                "pushl {0}", // r0
                "movl $0, %eax",
                "int $0x40",
                "addl  $16, %esp",

                in(reg) r0,

                // The following registers are clobbered by the syscall
                out("eax") _,
                out("ecx") _,
                out("edx") _,
                options(att_syntax),
            );
        }
    }

    // Yield 2 is used for yield_no_wait
    unsafe fn yield2([Register(r0), Register(r1)]: [Register; 2]) {
        unsafe {
            asm!(
                "pushl $0",
                "pushl $0",
                "pushl {0}", // r1
                "pushl {1}", // r0
                "movl  $0, %eax",
                "int $0x40",
                "addl  $16, %esp",

                in(reg) r1,
                in(reg) r0,

                // The following registers are clobbered by the syscall
                out("eax") _,
                out("ecx") _,
                out("edx") _,
                options(att_syntax)
            );
        }
    }

    unsafe fn syscall1<const CLASS: usize>([Register(mut r0)]: [Register; 1]) -> [Register; 2] {
        // This is memop, the only syscall class that syscall1 supports
        let r1;
        unsafe {
            asm!(
                "push $0",
                "push $0",
                "push $0",
                "push {0}", // r0
                "movl  $5, %eax",
                "int $0x40",
                "popl {0:e}", // r1
                "popl {1:e}", // r0
                "addl  $8, %esp",

                inlateout(reg) r0,
                out(reg) r1,

                // The following registers are clobbered by the syscall
                out("eax") _,
                options(att_syntax),
            );
        }
        [Register(r0), Register(r1)]
    }

    unsafe fn syscall2<const CLASS: usize>(
        [Register(mut r0), Register(mut r1)]: [Register; 2],
    ) -> [Register; 2] {
        let cmd: u32 = match CLASS {
            syscall_class::MEMOP => 5,
            syscall_class::EXIT => 6,
            _ => unreachable!(),
        };

        unsafe {
            asm!(
                "pushl $0",
                "pushl $0",
                "pushl {0}", // r1
                "pushl {1}", // r0
                "movl  {2}, %eax", // cmd
                "int $0x40",
                "popl {1:e}", // r0
                "popl {0:e}", // r1
                "addl  $8, %esp",

                inlateout(reg) r1,
                inlateout(reg) r0,
                in(reg) cmd,

                // The following registers are clobbered by the syscall
                out("eax") _,
                options(att_syntax),
            );
        }

        [Register(r0), Register(r1)]
    }

    unsafe fn syscall4<const CLASS: usize>(
        [Register(mut r0), Register(mut r1), Register(mut r2), Register(mut r3)]: [Register; 4],
    ) -> [Register; 4] {
        let cmd: u32 = match CLASS {
            syscall_class::SUBSCRIBE => 1,
            syscall_class::COMMAND => 2,
            syscall_class::ALLOW_RW => 3,
            syscall_class::ALLOW_RO => 4,
            _ => unreachable!(),
        };
        unsafe {
            asm!(
                "pushl {3}", // r3
                "pushl {2}", // r2
                "pushl {1}", // r1
                "pushl {0}", // r0
                "movl  {4:e}, %eax",
                "int $0x40",
                "popl {0:e}", // r0
                "popl {1:e}", // r1
                "popl {2:e}", // r2
                "popl {3:e}", // r3
                
                inlateout(reg) r0,
                inlateout(reg) r1,
                inlateout(reg) r2,
                inlateout(reg) r3,

                in(reg) cmd,

                // The following registers are clobbered by the syscall
                out("eax") _,
                options(att_syntax),
            );
        }

        [Register(r0), Register(r1), Register(r2), Register(r3)]
    }
}
