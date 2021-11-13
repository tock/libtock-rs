/// Calls the Exit system call when dropped. Used to catch panic unwinding from
/// a `#![no_std]` context. The caller should `core::mem::forget` the
/// `ExitOnDrop` when it no longer needs to catch unwinding.
///
///
/// # Example
/// ```
/// use libtock_platform::exit_on_drop::ExitOnDrop;
/// fn function_that_must_not_unwind<S: libtock_platform::Syscalls>() {
///     let exit_on_drop: ExitOnDrop::<S> = Default::default();
///     /* Do something that might unwind here. */
///     core::mem::forget(exit_on_drop);
/// }
/// ```
pub struct ExitOnDrop<S: crate::Syscalls>(core::marker::PhantomData<S>);

impl<S: crate::Syscalls> Default for ExitOnDrop<S> {
    fn default() -> ExitOnDrop<S> {
        ExitOnDrop(core::marker::PhantomData)
    }
}

impl<S: crate::Syscalls> Drop for ExitOnDrop<S> {
    fn drop(&mut self) {
        S::exit_terminate(0);
    }
}
