#![no_std]

use libtock_platform as platform;
use platform::{
    allow_rw, exit_on_drop, return_variant, share, subscribe, syscall_class, DefaultConfig,
    ErrorCode, Register, ReturnVariant, Syscalls, Upcall,
};

/// The IPC driver.
///
/// # Example
///
/// Service:
///
/// ```ignore
/// use libtock::ipc::{Ipc, IpcCallData, IpcListener};
/// use libtock::leds::Leds;
///
/// fn led_callback(data: IpcCallData) {
///     let _ = Leds::on(0);
/// }
///
/// // Creates an IPC service for turning on an LED
/// let listener = IpcListener(led_callback);
///
/// // Registers the IPC service
/// let _ = Ipc::register_service_listener(listener);
/// ```
///
/// Client:
///
/// ```ignore
/// use libtock::ipc::Ipc;
///
/// // Discovers the IPC service
/// let service_id = Ipc::discover("org.tockos.example.led").unwrap();
///
/// // Runs the IPC service
/// let _ = Ipc::notify_service(service_id);
/// ````

#[derive(Debug, Eq, PartialEq)]
pub struct IpcCallData<'a> {
    pub caller_id: u32,
    pub buffer: Option<&'a mut [u8]>,
}

pub struct Ipc<S: Syscalls, C: Config = DefaultConfig>(S, C);

impl<S: Syscalls, C: Config> Ipc<S, C> {
    /// Run a check against the IPC capsule to ensure it is present
    ///
    /// Returns Ok(()) if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::EXISTS, 0, 0).to_result()
    }

    /// Look up the service ID of an IPC service
    ///
    /// The package name provided should be the one indicated in the TBF header
    /// of the Tock binary presenting itself as an IPC service. For additional
    /// details, see the Tock Binary Format documentation here:
    ///
    /// <https://book.tockos.org/doc/tock_binary_format#3-package-name>
    pub fn discover(pkg_name: &[u8]) -> Result<u32, ErrorCode> {
        share::scope(|allow_search| {
            S::allow_ro::<C, DRIVER_NUM, { allow_ro::SEARCH }>(allow_search, pkg_name)?;
            S::command(DRIVER_NUM, command::DISCOVER, 0, 0).to_result()
        })
    }

    /// Register an IPC service
    ///
    /// This function is called by the IPC service to register a listener under
    /// a given package name. Only a single listener can be registered per
    /// package name. IPC clients can trigger this function to be executed by
    /// calling `Ipc::notify_service` with the current service's service ID.
    pub fn register_service_listener<F: Fn(IpcCallData)>(
        pkg_name: &[u8],
        listener: &'static IpcListener<F>,
    ) -> Result<(), ErrorCode> {
        let service_id = Self::discover(pkg_name)?;
        Self::subscribe_ipc::<C, _, DRIVER_NUM>(service_id, listener)
    }

    /// Register a client IPC callback
    ///
    /// This function is called by the IPC client to register a listener
    /// (callback) for a given IPC service, identified by its service ID. A
    /// single callback can be registered per service on each client. The
    /// corresponding service can trigger this callback using
    /// `Ipc::notify_slicent`, returning control to the user.
    pub fn register_client_listener<F: Fn(IpcCallData)>(
        service_id: u32,
        listener: &'static IpcListener<F>,
    ) -> Result<(), ErrorCode> {
        Self::subscribe_ipc::<C, _, DRIVER_NUM>(service_id, listener)
    }

    /// Notify an IPC service to run
    ///
    /// This function is called by the IPC client to trigger an IPC service
    /// to run. The service ID passed to this function is the same one that
    /// `Ipc::discover` returns.
    pub fn notify_service(service_id: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::SERVICE_NOTIFY, service_id, 0).to_result()
    }

    /// Notify a client IPC callback to run
    ///
    /// This function is called by the IPC service, generally as part of its
    /// listener, to trigger an IPC client callback to run. The client ID
    /// passed to this function is the same one that is presented in the
    /// `caller_id` field of the `IpcCallData` when the IPC service
    /// listener executes.
    pub fn notify_client(client_id: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::CLIENT_NOTIFY, client_id, 0).to_result()
    }

    /// Share a read/write buffer with an IPC service
    ///
    /// The client can call this function with a static mutable buffer in order
    /// to share it via a read/write allow with an IPC service. Since the
    /// buffer must be mutable and have static lifetime, the best way to
    /// create a reference to it is via a TakeCell (see the `takecell` crate).
    pub fn share(service_id: u32, buffer: &'static mut [u8]) -> Result<(), ErrorCode> {
        Self::allow_rw_ipc::<C, DRIVER_NUM>(service_id, buffer)
    }
}

/// A wrapper around a function to be registered and called when an IPC notify
/// is received.
///
/// The IPC API for registering listeners accepts static references to
/// instances of this struct, so in general the IPC function this struct
/// wraps should be an actual function instead of a short-lived closure.
pub struct IpcListener<F: Fn(IpcCallData)>(pub F);

impl<F: Fn(IpcCallData)> Upcall<subscribe::AnyId> for IpcListener<F> {
    fn upcall(&self, caller_id: u32, buffer_len: u32, buffer_ptr: u32) {
        let buffer_len = buffer_len as usize;
        let buffer = match buffer_len {
            0 => None,
            _ => {
                let buffer_addr = buffer_ptr as *mut u8;
                let buffer = unsafe { core::slice::from_raw_parts_mut(buffer_addr, buffer_len) };
                Some(buffer)
            }
        };
        self.0(IpcCallData { caller_id, buffer });
    }
}

// -----------------------------------------------------------------------------
// Implementation details below
// -----------------------------------------------------------------------------

impl<S: Syscalls, C: Config> Ipc<S, C> {
    fn subscribe_ipc<CONFIG: subscribe::Config, F: Fn(IpcCallData), const DRIVER_NUM: u32>(
        process_id: u32,
        listener: &'static IpcListener<F>,
    ) -> Result<(), ErrorCode> {
        // The upcall function passed to the Tock kernel.
        //
        // Safety: data must be a reference to a valid instance of Fn(IpcCallData).
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
        // monomorphized over DRIVER_NUM to keep code size small.
        //
        // Safety: upcall_fcn must be kernel_upcall<S, F> and upcall_data
        // must be a static reference to an instance of Fn(IpcCallData).
        unsafe fn inner<S: Syscalls, CONFIG: subscribe::Config>(
            driver_num: u32,
            subscribe_num: u32,
            upcall_fcn: Register,
            upcall_data: Register,
        ) -> Result<(), ErrorCode> {
            // Safety: syscall4's documentation indicates it can be used to
            // call Subscribe. These arguments follow TRD104. kernel_upcall has
            // the required signature. This function's preconditions mean that
            // upcall is a static reference to an instance of Fn(IpcCallData),
            // guaranteeing that upcall is still alive when kernel_upcall is
            // invoked.
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
        // Safety: upcall is a static reference to a Fn(IpcCallData) and will
        // therefore always be valid. upcall_fcn and upcall_data are derived in
        // ways that satisfy inner's requirements.
        unsafe { inner::<S, CONFIG>(DRIVER_NUM, process_id, upcall_fcn, upcall_data) }
    }

    fn allow_rw_ipc<CONFIG: allow_rw::Config, const DRIVER_NUM: u32>(
        buffer_num: u32,
        buffer: &'static mut [u8],
    ) -> Result<(), ErrorCode> {
        // Inner function that does the majority of the work. This is not
        // monomorphized over DRIVER_NUM and BUFFER_NUM to keep code size small.
        //
        // Safety: since `buffer` is a static reference, it will outlive
        // the actual allow, meaning a `Handle` is not needed as in
        // `libtock_platform::Syscalls::allow_rw`.
        unsafe fn inner<S: Syscalls, CONFIG: allow_rw::Config>(
            driver_num: u32,
            buffer_num: u32,
            buffer: &'static mut [u8],
        ) -> Result<(), ErrorCode> {
            // Safety: syscall4's documentation indicates it can be used to call
            // Read-Write Allow. These arguments follow TRD104.
            let [r0, r1, r2, _] = unsafe {
                S::syscall4::<{ syscall_class::ALLOW_RW }>([
                    driver_num.into(),
                    buffer_num.into(),
                    buffer.as_mut_ptr().into(),
                    buffer.len().into(),
                ])
            };

            let return_variant: ReturnVariant = r0.as_u32().into();
            // TRD 104 guarantees that Read-Write Allow returns either Success
            // with 2 U32 or Failure with 2 U32. We check the return variant by
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
            //      panicing.
            if return_variant == return_variant::FAILURE_2_U32 {
                // Safety: TRD 104 guarantees that if r0 is Failure with 2 U32,
                // then r1 will contain a valid error code. ErrorCode is
                // designed to be safely transmuted directly from a kernel error
                // code.
                return Err(unsafe { core::mem::transmute(r1.as_u32()) });
            }

            // r0 indicates Success with 2 u32s. Confirm a zero buffer was
            // returned, and it if wasn't then call the configured function.
            // We're relying on the optimizer to remove this branch if
            // returned_nozero_buffer is a no-op.
            let returned_buffer: (usize, usize) = (r1.into(), r2.into());
            if returned_buffer != (0, 0) {
                CONFIG::returned_nonzero_buffer(driver_num, buffer_num);
            }
            Ok(())
        }
        // Safety: since `allow_rw_ipc` never emits a call to `unallow_rw`
        // (unlike `libtock_platform::Syscalls::allow_rw`), we are guaranteed
        // that an AllowRw will always exist.
        unsafe { inner::<S, CONFIG>(DRIVER_NUM, buffer_num, buffer) }
    }
}

/// System call configuration trait for `Ipc`.
pub trait Config:
    platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config
{
}
impl<T: platform::allow_ro::Config + platform::allow_rw::Config + platform::subscribe::Config>
    Config for T
{
}

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

// Read-only allow numbers
mod allow_ro {
    pub const SEARCH: u32 = 0;
}
