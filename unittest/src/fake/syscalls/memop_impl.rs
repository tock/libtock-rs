//! `fake::Kernel`'s implementation of the Memop system call.

use crate::kernel_data::with_kernel_data;
use crate::{ExpectedSyscall, SyscallLogEntry};
use libtock_platform::{return_variant, ErrorCode, Register};
use std::convert::TryInto;

pub(super) fn memop(memop_num: Register, argument0: Register) -> [Register; 2] {
    let memop_num = memop_num.try_into().expect("Too large memop num");

    let (return_error, memop_return, memop_r1) = with_kernel_data(|option_kernel_data| {
        let kernel_data = option_kernel_data.expect("Memop called but no fake::Kernel exists");

        kernel_data.syscall_log.push(SyscallLogEntry::Memop {
            memop_num,
            argument0,
        });

        // Check for an expected syscall entry. Sets return_error to None if
        // the expected syscall queue is empty or if it expected this syscall
        // but did not specify a return override. Panics if a different syscall
        // was expected (either a non-Memop syscall, or a Memop call with
        // different arguments).
        let return_error = match kernel_data.expected_syscalls.pop_front() {
            None => None,
            Some(ExpectedSyscall::Memop {
                memop_num: expected_memop_num,
                argument0: expected_argument0,
                return_error,
            }) => {
                assert_eq!(
                    memop_num, expected_memop_num,
                    "expected different memop_num"
                );
                assert_eq!(
                    usize::from(argument0),
                    usize::from(expected_argument0),
                    "expected different argument0"
                );
                return_error
            }
            Some(expected_syscall) => expected_syscall.panic_wrong_call("Memop"),
        };

        // Emulate the memop call
        // TODO: This emulation could be improved by adding data to kernel_data to allow us to
        // better track what input arguments might be expected to return errors.
        let (memop_return, memop_r1) = match memop_num {
            0 => {
                /* brk */
                if Into::<*const u8>::into(argument0).is_null() {
                    (return_variant::FAILURE, ErrorCode::Invalid.into())
                } else {
                    kernel_data.memory_break = argument0.into();
                    (return_variant::SUCCESS, 0.into())
                }
            }
            1 => {
                /* sbrk */
                let current_brk = kernel_data.memory_break;
                let new_brk = current_brk.wrapping_byte_offset(argument0.as_i32() as isize);
                kernel_data.memory_break = new_brk;
                (return_variant::SUCCESS, kernel_data.memory_break.into())
            }
            2 => {
                /* app_ram_start */
                // just pick a random number to always return, for now
                (return_variant::SUCCESS, 0x123400.into())
            }
            10 => {
                /* debug_stack_start */
                (return_variant::SUCCESS, 0.into())
            }
            11 => {
                /* debug_heap_start */
                (return_variant::SUCCESS, 0.into())
            }
            _ => {
                panic!("Memop num not supported by test infrastructure");
            }
        };
        (return_error, memop_return, memop_r1)
    });

    // Convert the return value into the representative register values.
    // If there is an return_error, return a Failure along with that ErrorCode.
    let (return_variant, r1) = return_error.map_or((memop_return, memop_r1), |override_errcode| {
        (return_variant::FAILURE, override_errcode.into())
    });
    let r0: u32 = return_variant.into();
    [r0.into(), r1]
}
