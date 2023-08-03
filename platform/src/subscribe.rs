use crate::share::List;
use crate::ErrorCode;
use crate::Syscalls;

// -----------------------------------------------------------------------------
// `Subscribe` struct
// -----------------------------------------------------------------------------

/// A `Subscribe` instance allows safe code to call Tock's Subscribe system
/// call, by guaranteeing the upcall will be cleaned up before 'share ends. It
/// is generally used with the `share::scope` function, which offers a safe
/// interface for constructing `Subscribe` instances.
pub struct Subscribe<'share, S: Syscalls, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> {
    _syscalls: core::marker::PhantomData<S>,

    // Make this struct invariant with respect to the 'share lifetime.
    //
    // Covariance would be unsound, as that would allow code with a
    // `Subscribe<'static, ...>` to register an upcall that lasts for a shorter
    // lifetime, resulting in use-after-free if the upcall is invoked.
    // Contravariance would be sound, but is not necessary and may be confusing.
    //
    // Additionally, we want to have at least one private member of this struct
    // so that code outside this module cannot construct a `Subscribe` without
    // calling `ShareList::new`.
    _scope: core::marker::PhantomData<core::cell::Cell<&'share ()>>,
}

// We can't derive(Default) because S is not Default, and derive(Default)
// generates a Default implementation that requires S to be Default. Instead, we
// manually implement Default.
impl<'share, S: Syscalls, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> Default
    for Subscribe<'share, S, DRIVER_NUM, SUBSCRIBE_NUM>
{
    fn default() -> Self {
        Self {
            _syscalls: Default::default(),
            _scope: Default::default(),
        }
    }
}

impl<'share, S: Syscalls, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> Drop
    for Subscribe<'share, S, DRIVER_NUM, SUBSCRIBE_NUM>
{
    fn drop(&mut self) {
        S::unsubscribe(DRIVER_NUM, SUBSCRIBE_NUM);
    }
}

impl<'share, S: Syscalls, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> List
    for Subscribe<'share, S, DRIVER_NUM, SUBSCRIBE_NUM>
{
}

// -----------------------------------------------------------------------------
// `Upcall` trait
// -----------------------------------------------------------------------------

/// A Tock kernel upcall. Upcalls are registered using the Subscribe system
/// call, and are invoked during Yield calls.
///
/// Each `Upcall` supports one or more subscribe IDs, which are indicated by the
/// `SupportedIds` parameter. The types `AnySubscribeId` and `OneSubscribeId`
/// are provided to use as `SupportedIds` parameters in `Upcall`
/// implementations.
pub trait Upcall<SupportedIds> {
    fn upcall(&self, arg0: u32, arg1: u32, arg2: u32);
}

pub trait SupportsId<const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> {}

pub struct AnyId;
impl<const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> SupportsId<DRIVER_NUM, SUBSCRIBE_NUM>
    for AnyId
{
}

pub struct OneId<const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32>;
impl<const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> SupportsId<DRIVER_NUM, SUBSCRIBE_NUM>
    for OneId<DRIVER_NUM, SUBSCRIBE_NUM>
{
}

// -----------------------------------------------------------------------------
// Upcall implementations that simply store their arguments
// -----------------------------------------------------------------------------

/// An implementation of `Upcall` that sets the contained boolean value to
/// `true` when the upcall is invoked.
impl Upcall<AnyId> for core::cell::Cell<bool> {
    fn upcall(&self, _: u32, _: u32, _: u32) {
        self.set(true);
    }
}

/// Implemented for consistency with the other `Cell<Option<...>>` `Upcall`
/// impls. Most users would prefer the `Cell<bool>` implementation over this
/// impl, but this may be useful in a generic or macro context.
impl Upcall<AnyId> for core::cell::Cell<Option<()>> {
    fn upcall(&self, _: u32, _: u32, _: u32) {
        self.set(Some(()));
    }
}

/// An `Upcall` implementation that stores its first argument when called.
impl Upcall<AnyId> for core::cell::Cell<Option<(u32,)>> {
    fn upcall(&self, arg0: u32, _: u32, _: u32) {
        self.set(Some((arg0,)));
    }
}

/// An `Upcall` implementation that stores its first two arguments when called.
impl Upcall<AnyId> for core::cell::Cell<Option<(u32, u32)>> {
    fn upcall(&self, arg0: u32, arg1: u32, _: u32) {
        self.set(Some((arg0, arg1)));
    }
}

/// An `Upcall` implementation that stores its arguments when called.
impl Upcall<AnyId> for core::cell::Cell<Option<(u32, u32, u32)>> {
    fn upcall(&self, arg0: u32, arg1: u32, arg2: u32) {
        self.set(Some((arg0, arg1, arg2)));
    }
}

/// An `Upcall` implementation that interprets its first argument as a return
/// code.
impl Upcall<AnyId> for core::cell::Cell<Option<Result<(), ErrorCode>>> {
    fn upcall(&self, arg0: u32, _: u32, _: u32) {
        self.set(Some(match arg0 {
            0 => Ok(()),
            _ => Err(arg0.try_into().unwrap_or(ErrorCode::Fail)),
        }));
    }
}

/// An `Upcall` implementation that interprets its first argument as a return
/// code and stores its second argument when called.
impl Upcall<AnyId> for core::cell::Cell<Option<Result<(u32,), ErrorCode>>> {
    fn upcall(&self, arg0: u32, arg1: u32, _: u32) {
        self.set(Some(match arg0 {
            0 => Ok((arg1,)),
            _ => Err(arg0.try_into().unwrap_or(ErrorCode::Fail)),
        }));
    }
}

/// An `Upcall` implementation that interprets its first argument as a return
/// code and stores its second argument when called.
impl Upcall<AnyId> for core::cell::Cell<Option<Result<(u32, u32), ErrorCode>>> {
    fn upcall(&self, arg0: u32, arg1: u32, arg2: u32) {
        self.set(Some(match arg0 {
            0 => Ok((arg1, arg2)),
            _ => Err(arg0.try_into().unwrap_or(ErrorCode::Fail)),
        }));
    }
}

#[cfg(test)]
#[test]
fn upcall_impls() {
    let cell_bool = core::cell::Cell::new(false);
    cell_bool.upcall(1, 2, 3);
    assert!(cell_bool.get());

    let cell_empty = core::cell::Cell::new(None);
    cell_empty.upcall(1, 2, 3);
    assert_eq!(cell_empty.get(), Some(()));

    let cell_one = core::cell::Cell::new(None);
    cell_one.upcall(1, 2, 3);
    assert_eq!(cell_one.get(), Some((1,)));

    let cell_two = core::cell::Cell::new(None);
    cell_two.upcall(1, 2, 3);
    assert_eq!(cell_two.get(), Some((1, 2)));

    let cell_three = core::cell::Cell::new(None);
    cell_three.upcall(1, 2, 3);
    assert_eq!(cell_three.get(), Some((1, 2, 3)));

    let cell_returncode_empty_success: core::cell::Cell<Option<Result<(), ErrorCode>>> =
        core::cell::Cell::new(None);
    cell_returncode_empty_success.upcall(0, 2, 3);
    assert_eq!(cell_returncode_empty_success.get(), Some(Ok(())));
    let cell_returncode_empty_fail: core::cell::Cell<Option<Result<(), ErrorCode>>> =
        core::cell::Cell::new(None);
    cell_returncode_empty_fail.upcall(1, 2, 3);
    assert_eq!(cell_returncode_empty_fail.get(), Some(Err(ErrorCode::Fail)));

    let cell_returncode_one_success: core::cell::Cell<Option<Result<(u32,), ErrorCode>>> =
        core::cell::Cell::new(None);
    cell_returncode_one_success.upcall(0, 2, 3);
    assert_eq!(cell_returncode_one_success.get(), Some(Ok((2,))));
    let cell_returncode_one_fail: core::cell::Cell<Option<Result<(u32,), ErrorCode>>> =
        core::cell::Cell::new(None);
    cell_returncode_one_fail.upcall(1, 2, 3);
    assert_eq!(cell_returncode_one_fail.get(), Some(Err(ErrorCode::Fail)));

    let cell_returncode_two_success: core::cell::Cell<Option<Result<(u32, u32), ErrorCode>>> =
        core::cell::Cell::new(None);
    cell_returncode_two_success.upcall(0, 2, 3);
    assert_eq!(cell_returncode_two_success.get(), Some(Ok((2, 3))));
    let cell_returncode_two_fail: core::cell::Cell<Option<Result<(u32, u32), ErrorCode>>> =
        core::cell::Cell::new(None);
    cell_returncode_two_fail.upcall(1, 2, 3);
    assert_eq!(cell_returncode_two_fail.get(), Some(Err(ErrorCode::Fail)));
}

// -----------------------------------------------------------------------------
// `Config` trait
// -----------------------------------------------------------------------------

/// `Config` configures the behavior of the Subscribe system call. It should
/// generally be passed through by drivers, to allow application code to
/// configure error handling.
pub trait Config {
    /// Called if a Subscribe call succeeds and returns a non-null upcall. In
    /// some applications, this may indicate unexpected reentrance. By default,
    /// the non-null upcall is ignored.
    fn returned_nonnull_upcall(_driver_num: u32, _subscribe_num: u32) {}
}
