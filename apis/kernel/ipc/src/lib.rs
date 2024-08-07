#![no_std]

use core::slice::from_raw_parts_mut;
use libtock_platform as platform;
use libtock_platform::{
    exit_on_drop, return_variant, share, subscribe, syscall_class, DefaultConfig, ErrorCode,
    Register, ReturnVariant, Syscalls, Upcall,
};

pub struct Ipc<S: Syscalls, C: Config = DefaultConfig>(S, C);

#[derive(Debug, Eq, PartialEq)]
pub struct IpcCallData<'a> {
    caller_id: u32,
    buffer: &'a mut [u8],
}

impl<S: Syscalls, C: Config> Ipc<S, C> {
    /// Check if the IPC kernel driver exists
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::EXISTS, 0, 0).to_result()
    }

    /// Request the service ID of an IPC service by package name
    pub fn discover(pkg_name: &[u8]) -> Result<u32, ErrorCode> {
        share::scope(|allow_search| {
            // Share the package name buffer with the kernel to search for
            S::allow_ro::<C, DRIVER_NUM, { allow_ro::SEARCH }>(allow_search, pkg_name)?;

            // Send the command to the kernel driver to retrieve the service id for the
            // corresponding IPC service, if it exists
            S::command(DRIVER_NUM, command::DISCOVER, 0, 0).to_result()
        })
    }

    pub fn register_service_listener<F: Fn(IpcCallData)>(
        pkg_name: &[u8],
        listener: &'static IpcListener<F>,
    ) -> Result<(), ErrorCode> {
        let service_id = Self::discover(pkg_name)?;
        Self::subscribe_ipc::<C, _, DRIVER_NUM>(service_id, listener)
    }

    pub fn register_client_listener<F: Fn(IpcCallData)>(
        service_id: u32,
        listener: &'static IpcListener<F>,
    ) -> Result<(), ErrorCode> {
        Self::subscribe_ipc::<C, _, DRIVER_NUM>(service_id, listener)
    }

    pub fn notify_service(svc_id: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::SERVICE_NOTIFY, svc_id, 0).to_result()
    }

    pub fn notify_client(svc_id: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::CLIENT_NOTIFY, svc_id, 0).to_result()
    }

    fn subscribe_ipc<CONFIG: subscribe::Config, F: Fn(IpcCallData), const DRIVER_NUM: u32>(
        process_id: u32,
        listener: &'static IpcListener<F>,
    ) -> Result<(), ErrorCode> {
        // TODO: revise comments
        // The upcall function passed to the Tock kernel.
        //
        // Safety: data must be a reference to a valid instance of U.
        unsafe extern "C" fn kernel_upcall<S: Syscalls, F: Fn(IpcCallData)>(
            arg0: u32,
            arg1: u32,
            arg2: u32,
            data: Register,
        ) {
            let exit: exit_on_drop::ExitOnDrop<S> = Default::default();
            let upcall: *const IpcListener<F> = data.into();
            unsafe { &*upcall }.upcall(arg0, arg1, arg2);
            core::mem::forget(exit);
        }

        // Inner function that does the majority of the work. This is not
        // monomorphized over DRIVER_NUM and SUBSCRIBE_NUM to keep code size
        // small.
        //
        // Safety: upcall_fcn must be kernel_upcall<S, IDS, U> and upcall_data
        // must be a reference to an instance of U that will remain valid as
        // long as the 'scope lifetime is alive. Can only be called if a
        // Subscribe<'scope, S, driver_num, subscribe_num> exists.
        unsafe fn inner<S: Syscalls, CONFIG: subscribe::Config>(
            driver_num: u32,
            subscribe_num: u32,
            upcall_fcn: Register,
            upcall_data: Register,
        ) -> Result<(), ErrorCode> {
            // Safety: syscall4's documentation indicates it can be used to call
            // Subscribe. These arguments follow TRD104. kernel_upcall has the
            // required signature. This function's preconditions mean that
            // upcall is a reference to an instance of U that will remain valid
            // until the 'scope lifetime is alive The existence of the
            // Subscribe<'scope, Self, DRIVER_NUM, SUBSCRIBE_NUM> guarantees
            // that if this Subscribe succeeds then the upcall will be cleaned
            // up before the 'scope lifetime ends, guaranteeing that upcall is
            // still alive when kernel_upcall is invoked.
            let [r0, r1, _, _] = unsafe {
                S::syscall4::<{ syscall_class::SUBSCRIBE }>([
                    driver_num.into(),
                    subscribe_num.into(),
                    upcall_fcn,
                    upcall_data,
                ])
            };

            let return_variant: ReturnVariant = r0.as_u32().into();
            // TRD 104 guarantees that Subscribe returns either Success with 2
            // U32 or Failure with 2 U32. We check the return variant by
            // comparing against Failure with 2 U32 for 2 reasons:
            //
            //   1. On RISC-V with compressed instructions, it generates smaller
            //      code. FAILURE_2_U32 has value 2, which can be loaded into a
            //      register with a single compressed instruction, whereas
            //      loading SUCCESS_2_U32 uses an uncompressed instruction.
            //   2. In the event the kernel malfuctions and returns a different
            //      return variant, the success path is actually safer than the
            //      failure path. The failure path assumes that r1 contains an
            //      ErrorCode, and produces UB if it has an out of range value.
            //      Incorrectly assuming the call succeeded will not generate
            //      unsoundness, and will likely lead to the application
            //      hanging.
            if return_variant == return_variant::FAILURE_2_U32 {
                // Safety: TRD 104 guarantees that if r0 is Failure with 2 U32,
                // then r1 will contain a valid error code. ErrorCode is
                // designed to be safely transmuted directly from a kernel error
                // code.
                return Err(unsafe { core::mem::transmute(r1.as_u32()) });
            }

            // r0 indicates Success with 2 u32s. Confirm the null upcall was
            // returned, and it if wasn't then call the configured function.
            // We're relying on the optimizer to remove this branch if
            // returned_nonnull_upcall is a no-op.
            // Note: TRD 104 specifies that the null upcall has address 0,
            // not necessarily a null pointer.
            let returned_upcall: usize = r1.into();
            if returned_upcall != 0usize {
                CONFIG::returned_nonnull_upcall(driver_num, subscribe_num);
            }
            Ok(())
        }

        let upcall_fcn = (kernel_upcall::<S, F> as *const ()).into();
        let upcall_data = (listener as *const IpcListener<F>).into();
        // Safety: upcall's type guarantees it is a reference to a U that will
        // remain valid for at least the 'scope lifetime. _subscribe is a
        // reference to a Subscribe<'scope, Self, DRIVER_NUM, SUBSCRIBE_NUM>,
        // proving one exists. upcall_fcn and upcall_data are derived in ways
        // that satisfy inner's requirements.
        unsafe { inner::<S, CONFIG>(DRIVER_NUM, process_id, upcall_fcn, upcall_data) }
    }
}

/// A wrapper around a static function to be registered as an IPC callback
/// and called when the IPC service receives a notify.
///
/// ```ignore
/// fn run_on_notify(caller_id: u32, buffer: &mut [u8]) {
///     // make use of the caller ID and shared buffer
/// }
///
/// let callback = Ipc(run_on_notify);
/// ```
pub struct IpcListener<F: Fn(IpcCallData)>(pub F);

impl<F: Fn(IpcCallData)> Upcall<subscribe::AnyId> for IpcListener<F> {
    fn upcall(&self, caller_id: u32, buffer_len: u32, buffer_ptr: u32) {
        // Safety: TODO
        let buffer: &mut [u8] =
            unsafe { from_raw_parts_mut::<u8>(buffer_ptr as *mut u8, buffer_len as usize) };
        self.0(IpcCallData { caller_id, buffer });
    }
}

/// System call configuration trait for `Ipc`.
pub trait Config: platform::allow_ro::Config + platform::subscribe::Config {}
impl<T: platform::allow_ro::Config + platform::subscribe::Config> Config for T {}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x10000;

// Command IDs
mod command {
    pub const EXISTS: u32 = 0;
    pub const DISCOVER: u32 = 1;
    pub const SERVICE_NOTIFY: u32 = 2;
    pub const CLIENT_NOTIFY: u32 = 3;
}

mod allow_ro {
    pub const SEARCH: u32 = 0;
}
