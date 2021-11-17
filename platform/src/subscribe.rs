use crate::syscall_scope::ShareList;
use crate::Syscalls;

// -----------------------------------------------------------------------------
// `Subscribe` struct
// -----------------------------------------------------------------------------

/// A `Subscribe` instance allows safe code to call Tock's Subscribe system
/// call, by guaranteeing the upcall will be cleaned up before 'scope ends. It
/// is generally used with the `syscall_scope` function, which offers a safe
/// interface for constructing `Subscribe` instances.
pub struct Subscribe<'scope, S: Syscalls, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> {
    _syscalls: core::marker::PhantomData<S>,

    // Make this struct invariant with respect to the 'scope lifetime.
    //
    // Covariance would be unsound, as that would allow code with a
    // `Subscribe<'static, ...>` to register an upcall that lasts for a shorter
    // lifetime, resulting in use-after-free if the upcall is invoked.
    // Contravariance would be sound, but is not necessary and may be confusing.
    //
    // Additionally, we want to have at least one private member of this struct
    // so that code outside this module cannot construct a `Subscribe` without
    // calling `ShareList::new`.
    _scope: core::marker::PhantomData<core::cell::Cell<&'scope ()>>,
}

impl<'scope, S: Syscalls, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> Drop
    for Subscribe<'scope, S, DRIVER_NUM, SUBSCRIBE_NUM>
{
    fn drop(&mut self) {
        S::unsubscribe(DRIVER_NUM, SUBSCRIBE_NUM);
    }
}

impl<'scope, S: Syscalls, const DRIVER_NUM: u32, const SUBSCRIBE_NUM: u32> ShareList<'scope>
    for Subscribe<'scope, S, DRIVER_NUM, SUBSCRIBE_NUM>
{
    // Safety: The safety invariant is inherited from the ShareList trait's
    // definition. The caller must guarantee that Drop::drop is called on this
    // Subscribe before the 'scope lifetime ends.
    unsafe fn new() -> Subscribe<'scope, S, DRIVER_NUM, SUBSCRIBE_NUM> {
        Subscribe {
            _syscalls: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
        }
    }
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
