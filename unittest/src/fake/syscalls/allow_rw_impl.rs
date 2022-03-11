use crate::kernel_data::with_kernel_data;
use crate::{ExpectedSyscall, SyscallLogEntry};
use libtock_platform::{return_variant, ErrorCode, Register};
use std::convert::TryInto;

pub(super) unsafe fn allow_rw(
    driver_num: Register,
    buffer_num: Register,
    address: Register,
    len: Register,
) -> [Register; 4] {
    let driver_num = driver_num.try_into().expect("Too large driver number");
    let buffer_num = buffer_num.try_into().expect("Too large buffer number");
    let result = with_kernel_data(|option_kernel_data| {
        let kernel_data =
            option_kernel_data.expect("Read-Write Allow called but no fake::Kernel exists");

        kernel_data.syscall_log.push(SyscallLogEntry::AllowRw {
            driver_num,
            buffer_num,
            len: len.into(),
        });

        // Check for an expected syscall entry. Returns an error from the lambda
        // if this syscall was expected and return_error was specified. Panics
        // if a different syscall was expected.
        match kernel_data.expected_syscalls.pop_front() {
            None => {}
            Some(ExpectedSyscall::AllowRw {
                driver_num: expected_driver_num,
                buffer_num: expected_buffer_num,
                return_error,
            }) => {
                assert_eq!(
                    driver_num, expected_driver_num,
                    "expected different driver_num"
                );
                assert_eq!(
                    buffer_num, expected_buffer_num,
                    "expected different buffer_num"
                );
                if let Some(error_code) = return_error {
                    return Err(error_code);
                }
            }
            Some(expected_syscall) => expected_syscall.panic_wrong_call("Read-Write Allow"),
        };

        let driver = match kernel_data.drivers.get(&driver_num) {
            None => return Err(ErrorCode::NoDevice),
            Some(driver_data) => driver_data.driver.clone(),
        };

        // Safety: RawSyscall requires the caller to specify address and len as
        // required by TRD 104. That trivially satisfies the precondition of
        // insert_rw_buffer, which also requires address and len to follow TRD
        // 104.
        let buffer = unsafe { kernel_data.allow_db.insert_rw_buffer(address, len) }.expect(
            "Read-Write Allow called with a buffer that overlaps an already-Allowed buffer",
        );

        Ok((driver, buffer))
    });

    let (driver, buffer) = match result {
        Ok((driver, buffer)) => (driver, buffer),
        Err(error_code) => {
            let r0: u32 = return_variant::FAILURE_2_U32.into();
            let r1: u32 = error_code as u32;
            return [r0.into(), r1.into(), address, len];
        }
    };

    let (error_code, buffer_out) = match driver.allow_readwrite(buffer_num, buffer) {
        Ok(buffer_out) => (None, buffer_out),
        Err((buffer_out, error_code)) => (Some(error_code), buffer_out),
    };

    let (address_out, len_out) = with_kernel_data(|option_kernel_data| {
        let kernel_data = option_kernel_data
            .expect("fake::Kernel dropped during fake::SyscallDriver::allow_readwrite");
        kernel_data.allow_db.remove_rw_buffer(buffer_out)
    });

    match error_code {
        None => {
            let r0: u32 = return_variant::SUCCESS_2_U32.into();
            // The value of r3 isn't specified in TRD 104, but in practice the
            // kernel won't change it. This mimics that behavior, for lack of a
            // better option.
            [r0.into(), address_out, len_out, len]
        }
        Some(error_code) => {
            let r0: u32 = return_variant::FAILURE_2_U32.into();
            let r1: u32 = error_code as u32;
            [r0.into(), r1.into(), address_out, len_out]
        }
    }
}
