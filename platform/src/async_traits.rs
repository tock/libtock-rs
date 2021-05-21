//! Traits for building lightweight asynchronous APIs. These traits lack the
//! dynamic capabilities of `core::future::Future`, but have much smaller code
//! size and RAM usage costs.
//!
//! Tock kernel developers will be familiar with the 2-phase pattern of
//! operation these traits support. Client code (code using an asynchronous
//! component) calls a function or method provided by the asynchronous component
//! to start an asynchronous operation. When the operation is complete, the
//! asynchronous component calls a callback defined using the `FreeCallback`
//! and/or `MethodCallback` traits.
//!
//! Note that asynchronous callbacks must only be called from within "callback
//! context" -- that is, within a kernel callback (registered using the
//! `subscribe` system call). To enforce that, the callback traits require the
//! `CallbackContext` type, which is only instantiated by an instance of the
//! `Syscalls` trait.
//!
//! There is no prohibition on a callback calling back into the asynchronous
//! component that called it. In other words, if component B calls
//! `<A as FreeCallback<Done>>::call(...)`, then `call()` can call back into B
//! to start a new operation. Asynchronous components should clean up their
//! state internally before calling callbacks so as to support reentrant calls
//! into themselves!
//!
//! In client code, prefer to implement `FreeCallback` instead of
//! `MethodCallback` when possible, as it is easier to pass a `FreeCallback` to
//! an asynchronous component.
//!
//! To bridge the gap between `FreeCallback` and `MethodCallback`, we also have
//! the `Locator` trait. `Locator` allows `FreeCallback` implementations to find
//! their global state, and also provides `FreeCallback` versions of callbacks
//! for types that implement `MethodCallback`. In general, `Locator` should be
//! implemented by `libtock_runtime::static_component!` (in real Tock apps) and
//! `libtock_unittest::test_component!` (in unit tests), rather than directly by
//! user code.

/// `FreeCallback` is the callback equivalent of a free function: it does not
/// have access to the client component's data. `FreeCallback` is used by
/// asynchronous components -- such as `Syscalls` -- which cannot efficiently
/// store a client reference.
pub trait FreeCallback<AsyncResponse> {
    fn call(context: CallbackContext, response: AsyncResponse);
}

/// `MethodCallback` is a callback method; it can access the client component's
/// data. Note that asynchronous components generally need to use interior
/// mutability to mutate data, as `MethodCallback` is designed under the
/// assumption that there are multiple references to most asynchronous
/// components at any given time.
pub trait MethodCallback<AsyncResponse> {
    fn call(&self, context: CallbackContext, response: AsyncResponse);
}

/// `Syscalls` instantiates a `CallbackContext` when a kernel callback is
/// called. The lifetime prevents the `CallbackContext` from being copied into
/// storage that outlives the callback. Code that is only safe to call from
/// callback context can request a `CallbackContext` argument. `CallbackContext`
/// is a zero-sized type so passing it around has no runtime cost.
#[derive(Clone, Copy)]
pub struct CallbackContext<'c> {
    // `_phantom` serves three purposes. It uses the `c lifetime to avoid an
    // "unused lifetime" error, it provides the proper variance over 'c, and it
    // prevents code outside this crate from directly constructing a
    // CallbackContext (because of its visibility control). Code outside this
    // crate can copy a CallbackContext, but that is fine as copying the
    // CallbackContext preserves its associated lifetime.
    pub(crate) _phantom: core::marker::PhantomData<&'c ()>,
}

/// Provides access to a global instance of type `Target`. Every call to
/// `locate()` on a given Locator type should return a reference to the same
/// instance of `Target`. An instance of `Locator` generally isn't instantiated
/// directly; instead, its type is passed to where it is needed via generic
/// arguments.
///
/// For convenience, Locator provides a `FreeCallback` implementation for every
/// `MethodCallback` implementation that `Target` has.
pub trait Locator: 'static {
    type Target;
    fn locate() -> &'static Self::Target;
}

impl<L: Locator, AsyncResponse> FreeCallback<AsyncResponse> for L
where
    L::Target: MethodCallback<AsyncResponse>,
{
    fn call(context: CallbackContext, response: AsyncResponse) {
        L::locate().call(context, response);
    }
}
