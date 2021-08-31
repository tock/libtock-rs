use crate::callback::CallbackSubscription;
use crate::callback::Consumer;
use crate::result::AllowError;
use crate::result::CommandError;
use crate::result::SubscribeError;
use crate::shared_memory::SharedMemory;

use libtock_platform::{RawSyscalls, Syscalls};
use libtock_runtime::TockSyscalls;

pub mod raw {
    use libtock_platform::Syscalls;
    use libtock_runtime::TockSyscalls;
    pub unsafe fn memop(arg0: usize, arg1: usize) -> isize {
        TockSyscalls::memop(arg0, arg1)
    }
    pub fn yield_wait() {
        TockSyscalls::yield_wait();
    }
}

pub fn subscribe<C: Consumer<T>, T>(
    driver_number: usize,
    subscribe_number: usize,
    payload: &mut T,
) -> Result<CallbackSubscription, SubscribeError> {
    extern "C" fn c_callback<T, C: Consumer<T>>(
        arg1: usize,
        arg2: usize,
        arg3: usize,
        data: usize,
    ) {
        let payload = unsafe { &mut *(data as *mut T) };
        C::consume(payload, arg1, arg2, arg3);
    }

    subscribe_fn(
        driver_number,
        subscribe_number,
        c_callback::<T, C>,
        payload as *mut _ as usize,
    )
    .map(|_| CallbackSubscription::new(driver_number, subscribe_number))
}

const TOCK_SYSCALL_SUCCESS_U32_U32: u32 = 130;
const TOCK_SYSCALL_FAILURE_U32_U32: u32 = 2;

pub fn subscribe_fn(
    driver_number: usize,
    subscribe_number: usize,
    callback: extern "C" fn(usize, usize, usize, usize),
    userdata: usize,
) -> Result<(), SubscribeError> {
    let [r0, _r1, _r2, r3] = unsafe {
        TockSyscalls::syscall4::<1>([
            (driver_number as u32).into(),
            (subscribe_number as u32).into(),
            (callback as *const u32).into(),
            userdata.into(),
        ])
    };
    match r0.as_u32() {
        TOCK_SYSCALL_SUCCESS_U32_U32 => {
            //for now, ignore the returned callback / userdata, same as old iface
            Ok(())
        }
        TOCK_SYSCALL_FAILURE_U32_U32 => Err(SubscribeError {
            driver_number,
            subscribe_number,
            return_code: r3.as_u32() as isize * -1,
        }),
        _ => panic!("BADRVAL"),
    }
}

pub fn command_noval(
    driver_number: usize,
    command_number: usize,
    arg1: usize,
    arg2: usize,
) -> Result<(), CommandError> {
    TockSyscalls::command(
        driver_number as u32,
        command_number as u32,
        arg1 as u32,
        arg2 as u32,
    )
    .get_success_or_failure()
    .map_err(|e| CommandError {
        driver_number,
        command_number,
        arg1,
        arg2,
        return_code: (e as isize) * -1,
    })
}

pub fn command_u32(
    driver_number: usize,
    command_number: usize,
    arg1: usize,
    arg2: usize,
) -> Result<u32, CommandError> {
    TockSyscalls::command(
        driver_number as u32,
        command_number as u32,
        arg1 as u32,
        arg2 as u32,
    )
    .get_success_u32_or_failure()
    .map_err(|e| CommandError {
        driver_number,
        command_number,
        arg1,
        arg2,
        return_code: (e as isize) * -1,
    })
}

pub fn allow_readwrite(
    driver_number: usize,
    allow_number: usize,
    buffer_to_share: &mut [u8],
) -> Result<SharedMemory, AllowError> {
    let len = buffer_to_share.len();
    let [r0, _r1, _r2, r3] = unsafe {
        TockSyscalls::syscall4::<3>([
            driver_number.into(),
            allow_number.into(),
            (buffer_to_share.as_mut_ptr() as u32).into(),
            len.into(),
        ])
    };
    match r0.as_u32() {
        TOCK_SYSCALL_SUCCESS_U32_U32 => {
            // for now, continue to drop whatever is returned?
            Ok(SharedMemory::new(
                driver_number,
                allow_number,
                buffer_to_share,
            ))
        }
        TOCK_SYSCALL_FAILURE_U32_U32 => Err(AllowError {
            driver_number,
            allow_number,
            return_code: r3.as_u32() as isize * -1,
        }),
        _ => panic!("BADRVAL"),
    }
}

// For now, just drop the returned buffer. Caller maintains read access
// to the shared slice anyway. Problem is, we really should have a way to
// indicate the lifetime of this borrow...
pub fn allow_readonly(
    driver_number: usize,
    allow_number: usize,
    buffer_to_share: &[u8],
) -> Result<(), AllowError> {
    let len = buffer_to_share.len();
    let [r0, _r1, _r2, r3] = unsafe {
        TockSyscalls::syscall4::<4>([
            driver_number.into(),
            allow_number.into(),
            (buffer_to_share.as_ptr() as u32).into(),
            len.into(),
        ])
    };
    match r0.as_u32() {
        TOCK_SYSCALL_SUCCESS_U32_U32 => {
            // for now, continue to drop whatever is returned?
            Ok(())
        }
        // shared error with rw allow for now
        TOCK_SYSCALL_FAILURE_U32_U32 => Err(AllowError {
            driver_number,
            allow_number,
            return_code: r3.as_u32() as isize * -1,
        }),
        _ => panic!("BADRVAL"),
    }
}
